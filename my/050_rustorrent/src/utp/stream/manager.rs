use std::{
    sync::{
        atomic::{
            Ordering
        }
    },
    time::{
        Instant, 
        Duration
    }
};
use async_std::{
    sync::{
        Arc, 
        channel, 
        Sender, 
        Receiver
    },
    io::{
        ErrorKind, 
        Error
    },
    net::{
        SocketAddr, 
        UdpSocket
    },
    task
};
use fixed::{
    types::{
        I48F16
    }
};
use shared_arena::{
    ArenaBox, 
    SharedArena
};
use crate::{
    utp::{
        delay_history::{
            DelayHistory,
        },
        sequence_number::{
            SequenceNumber
        },
        packet::{
            Packet,
        },
        header::{
            HEADER_SIZE
        },
        constants::{
            UDP_IPV4_MTU, 
            UDP_IPV6_MTU,
            MSS,
            TARGET, 
            MIN_CWND
        },
        UtpError, 
        Result, 
        Timestamp, 
        PacketType, 
        SelectiveAckBit, 
        State as UtpState
    }
};
use super::{
    state::{
        State
    },
    event::{
        UtpEvent
    },
    stream::{
        UtpStream
    },
    writer::{
        WriterCommand,
        WriterUserCommand,
        UtpWriter
    }
};

#[derive(Debug)]
pub struct UtpManager {
    /// Сокет
    socket: Arc<UdpSocket>,

    /// Канал получения новых событий
    recv: Receiver<UtpEvent>,

    /// Не дожидаемся блокирования состояния
    /// Ожидание может привести к дедлоку состояния
    state: Arc<State>,

    /// Адрес сокета
    addr: SocketAddr,

    /// Канал отправки данных пользователю
    writer_user: Sender<WriterUserCommand>,

    /// Канал отправки данных писателю
    writer: Sender<WriterCommand>,

    /// Пулл пакетов + атомарный счетчик ссылок сверху
    packet_arena: Arc<SharedArena<Packet>>,

    /// Время таймаута
    timeout: Instant,

    /// Количество последовательных раз, когда произошел таймаут
    /// После 3х раз мы закрываем сокет
    ntimeout: usize,

    /// Канал для коллбека об успешном подключении
    on_connected: Option<Sender<bool>>,

    /// Количество дубликаций ack??
    ack_duplicate: u8,
    /// Номер последнего ACK
    last_ack: SequenceNumber,

    delay_history: DelayHistory,

    /// Список утерянных пакетов
    tmp_packet_losts: Vec<SequenceNumber>,

    /// Сколько всего потеряли пакетов
    nlost: usize,

    slow_start: bool,

    next_cwnd_decrease: Instant,

    rtt: usize,
    rtt_var: usize,
    rto: usize
}

impl Drop for UtpManager {
    fn drop(&mut self) {
        println!("DROP UTP MANAGER", );
        // При вызове уничтожения менеджера мы можем отправить сообщение об успешном
        // отсоединении
        if let Some(on_connected) = self.on_connected.take() {
            // Создаем специально новую корутину, так как эта функция не async
            task::spawn(async move {
                on_connected.send(false).await
            });
        };
    }
}

impl UtpManager {
    pub fn new(socket: Arc<UdpSocket>, 
               addr: SocketAddr,
               recv: Receiver<UtpEvent>,
               packet_arena: Arc<SharedArena<Packet>>) -> UtpManager {
        Self::new_with_state(socket, addr, recv, Default::default(), packet_arena)
    }

    /// Новый менеджер с конкретным состоянием
    pub fn new_with_state(socket: Arc<UdpSocket>,
                          addr: SocketAddr,
                          recv: Receiver<UtpEvent>,
                          state: Arc<State>,
                          packet_arena: Arc<SharedArena<Packet>>) -> UtpManager {

        // Канал записи команд
        let (writer, writer_rcv) = channel(10);
        // Канал для пользовательских команд
        let (writer_user, writer_user_rcv) = channel(10);

        // Создаем актора записи комманд в сокет
        let writer_actor = UtpWriter::new(socket.clone(), 
                                          addr, 
                                          writer_user_rcv,
                                          writer_rcv, 
                                          Arc::clone(&state), 
                                          packet_arena.clone());
        // Стартуем его
        task::spawn(async move {
            writer_actor.start().await;
        });

        // Создаем менеджер
        UtpManager {
            socket,
            addr,
            recv,
            writer,
            state,
            writer_user,
            packet_arena,
            timeout: Instant::now() + Duration::from_secs(3),
            ntimeout: 0,
            on_connected: None,
            last_ack: SequenceNumber::zero(),
            ack_duplicate: 0,
            delay_history: DelayHistory::new(),
            tmp_packet_losts: Vec::new(),
            nlost: 0,
            slow_start: true,
            next_cwnd_decrease: Instant::now(),
            rtt: 0,
            rtt_var: 0,
            rto: 0
        }
    }

    /// Устанавливаем канал для успешного подключения
    pub fn set_on_connected(&mut self, on_connected: Sender<bool>) {
        self.on_connected = Some(on_connected);
    }

    /// Получаем поток данных для данного менеджера
    pub fn get_stream(&self) -> UtpStream {
        UtpStream {
            // reader_command: self.reader.clone(),
            // reader_result: self._reader_result_rcv.clone(),
            writer_user_command: self.writer_user.clone()
        }
    }

    /// Цикл работы с сокетом
    pub async fn start(mut self) -> Result<()> {
        // Устанавливаем соединение
        self.ensure_connected().await;
        
        // Разбираем в цикле входящие события
        while let Ok(incoming) = self.recv.recv().await {
            self
                .process_incoming(incoming)
                .await
                .unwrap();
        }
        Ok(())
    }

    /// Дожидаемся установки соединения
    async fn ensure_connected(&self) {
        // Получаем состояние
        let state = self.state.utp_state();

        // Если мы уже подключены - выходим
        if state != UtpState::MustConnect {
            return;
        }

        // Отправляем пакет SYN
        self.writer.send(WriterCommand::SendPacket {
            packet_type: PacketType::Syn
        }).await;
    }

    /// Обработка прилетевшего сообщенияы
    async fn process_incoming(&mut self, event: UtpEvent) -> Result<()> {
        match event {
            // Входящий пакет
            UtpEvent::IncomingPacket { packet } => {
                // Обрабатываем
                match self.dispatch(packet).await {
                    // Если потеря пакеты выявлена
                    Err(UtpError::PacketLost) => {
                        // Тогда отправляем сообщение о потере пакета
                        self.writer.send(WriterCommand::ResendPacket {
                            only_lost: true
                        }).await;
                    }
                    Ok(_) => {
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            UtpEvent::Tick => {
                // Если прошло много времени - обрабатываем таймаут
                if Instant::now() > self.timeout {
                    self.on_timeout().await?;
                } else {
                    // println!("TICK RECEVEID BUT NOT TIMEOUT {:?} {:?}", self.timeout, self.rto);
                }
            }
        }
        Ok(())
    }

    /// Обработка таймаута
    async fn on_timeout(&mut self) -> Result<()> {
        use UtpState::*;

        // Получаем состояние
        let utp_state = self.state.utp_state();

        // Если количество таймаутом больше 3х и режим отправки syn или таймаутов больше 7
        if ((utp_state == SynSent) && (self.ntimeout >= 3)) || 
            (self.ntimeout >= 7) {
            return Err(Error::new(ErrorKind::TimedOut, "utp timed out").into());
        }

        // Проверяем какое сейчас состояние
        match utp_state {
            // Если отправки SYN
            SynSent => {
                // Тогда отсылаем SYN
                self.writer.send(WriterCommand::SendPacket { 
                    packet_type: PacketType::Syn 
                }).await;
            }
            // Если мы уже соединены - тогда отсылаем незавершенные пакеты еще раз
            Connected => {
                if self.state.inflight_size() > 0 {
                    self.writer.send(WriterCommand::ResendPacket { 
                        only_lost: false 
                    }).await;
                }
            }
            _ => {}
        }

        if self.state.inflight_size() > 0 {
            self.ntimeout += 1;
        } else {
            println!("NLOST {:?}", self.nlost);
            // println!("RTO {:?} RTT {:?} RTT_VAR {:?}", self.rto, self.rtt, self.rtt_var);
        }

        self.timeout = Instant::now() + Duration::from_millis(self.rto as u64);

        Ok(())
    }

    /// Обработка пакета, ArenaBox - значи объект из пула объектов
    async fn dispatch(&mut self, packet: ArenaBox<Packet>) -> Result<()> {
        // println!("DISPATCH HEADER: {:?}", packet.header());

        // Какая была задержка?
        let _delay = packet.received_at().delay(packet.get_timestamp());
        // self.delay = Delay::since(packet.get_timestamp());

        let utp_state = self.state.utp_state();

        self.ntimeout = 0;
        self.timeout = Instant::now() + Duration::from_millis(self.rto as u64);

        // Проверяем тип пакета и текущее состояние
        match (packet.get_type()?, utp_state) {
            // Пакет синка, а состояния нету еще
            (PacketType::Syn, UtpState::None) => {
                //println!("RECEIVED SYN {:?}", self.addr);
                let connection_id = packet.get_connection_id();

                // Выставляем подключенное состояние
                self.state.set_utp_state(UtpState::Connected);
                // Выставляем айдишник отправки и получения (+1)
                self.state.set_recv_id(connection_id + 1);
                self.state.set_send_id(connection_id);
                // Рандомный ноер последовательности
                self.state.set_seq_number(SequenceNumber::random());
                self.state.set_ack_number(packet.get_seq_number());
                self.state.set_remote_window(packet.get_window_size());

                // Отправляем пакет состояния
                self.writer.send(WriterCommand::SendPacket {
                    packet_type: PacketType::State
                }).await;
            }
            (PacketType::Syn, _) => {
            }
            // Если получили пакет состояния
            (PacketType::State, UtpState::SynSent) => {
                // Related:
                // https://engineering.bittorrent.com/2015/08/27/drdos-udp-based-protocols-and-bittorrent/
                // https://www.usenix.org/system/files/conference/woot15/woot15-paper-adamsky.pdf
                // https://github.com/bittorrent/libutp/commit/13d33254262d46b638d35c4bc1a2f76cea885760

                self.state.set_utp_state(UtpState::Connected);
                self.state.set_ack_number(packet.get_seq_number() - 1);
                self.state.set_remote_window(packet.get_window_size());

                if let Some(notify) = self.on_connected.take() {
                    notify.send(true).await;
                };

                self.state.remove_packets(packet.get_ack_number()).await;
            }
            (PacketType::State, UtpState::Connected) => {
                let remote_window = self.state.remote_window();

                if remote_window != packet.get_window_size() {
                    panic!("WINDOW SIZE CHANGED {:?}", packet.get_window_size());
                }

                self.handle_state(packet).await?;
            }
            (PacketType::State, _) => {
                // Wrong Packet
            }
            (PacketType::Data, _) => {
            }
            (PacketType::Fin, _) => {
            }
            (PacketType::Reset, _) => {
            }
        }

        Ok(())
    }


    async fn handle_state(&mut self, packet: ArenaBox<Packet>) -> Result<()> {
        let ack_number = packet.get_ack_number();
        let received_at = packet.received_at();

        // println!("ACK RECEIVED {:?} LAST_ACK {:?} DUP {:?} INFLIGHT {:?}",
        //          ack_number, self.last_ack, self.ack_duplicate, self.state.inflight_size());

        let in_flight = self.state.inflight_size();
        let mut bytes_newly_acked = 0;

        // let before = self.state.inflight_size();
        bytes_newly_acked += self.ack_packets(ack_number, received_at).await;
        // bytes_newly_acked += self.state.remove_packets(ack_number).await;
        //println!("PACKETS IN FLIGHT {:?}", self.inflight_packets.len());
        //        println!("PACKETS IN FLIGHT {:?}", self.inflight_packets.as_slices());

        // println!("BEFORE {:?} AFTER {:?}", before, self.state.inflight_size());

        let mut lost = false;

        if self.last_ack == ack_number {
            self.ack_duplicate = self.ack_duplicate.saturating_add(1);
            if self.ack_duplicate >= 3 {
                self.tmp_packet_losts.push(ack_number + 1);
                lost = true;
            }
        } else {
            self.ack_duplicate = 0;
            self.last_ack = ack_number;
        }

        if packet.has_extension() {
            //println!("HAS EXTENSIONS !", );
            for select_ack in packet.iter_sacks() {
                lost = select_ack.has_missing_ack() || lost;
                //println!("SACKS ACKED: {:?}", select_ack.nackeds());
                for ack_bit in select_ack {
                    match ack_bit {
                        SelectiveAckBit::Missing(seq_num) => {
                            self.tmp_packet_losts.push(seq_num);
                        }
                        SelectiveAckBit::Acked(seq_num) => {
                            bytes_newly_acked += self.ack_one_packet(seq_num, received_at).await;
//                            bytes_newly_acked += self.state.remove_packet(seq_num).await;
                        }
                    }
                }
            }
        }

        let delay = packet.get_timestamp_diff();
        if !delay.is_zero() {
            self.delay_history.add_delay(delay);
            if !lost {
                self.apply_congestion_control(bytes_newly_acked, in_flight);
            }
        }

        if lost {
            let now = Instant::now();

            if self.next_cwnd_decrease < now {
                let cwnd = self.state.cwnd();
                let cwnd = cwnd.min((cwnd / 2).max(MIN_CWND * MSS));
                self.state.set_cwnd(cwnd);
                self.next_cwnd_decrease = now + Duration::from_millis(100);
            }

            if self.slow_start {
                println!("STOP SLOW START AT {:?}", self.state.cwnd());
                self.slow_start = false;
            }

            self.nlost += self.tmp_packet_losts.len();

            self.mark_packets_as_lost().await;
            // println!("MISSING FROM SACK {:?}", self.lost_packets);
            return Err(UtpError::PacketLost);
        }

        self.rto = 500.max(self.rtt + self.rtt_var * 4);

        self.writer.send(WriterCommand::Acks).await;

        Ok(())
    }

    fn update_rtt(
        packet: &Packet,
        ack_received_at: Timestamp,
        mut rtt: usize,
        mut rtt_var: usize
    ) -> (usize, usize)
    {
        if !packet.resent {
            let rtt_packet = packet.get_timestamp().elapsed_millis(ack_received_at) as usize;

            if rtt == 0 {
                rtt = rtt_packet;
                rtt_var = rtt_packet / 2;
            } else {
                let delta: i64 = rtt as i64 - rtt_packet as i64;
                rtt_var = (rtt_var as i64 + (delta.abs() - rtt_var as i64) / 4) as usize;
                rtt = rtt - (rtt / 8) + (rtt_packet / 8);
            }
        }
        (rtt, rtt_var)
    }

    async fn ack_packets(&mut self, ack_number: SequenceNumber, ack_received_at: Timestamp) -> usize {
        // We need to make temporary vars to make the borrow checker happy
        let (mut rtt, mut rtt_var) = (self.rtt, self.rtt_var);

        let mut size = 0;
        let mut inflight_packets = self.state.inflight_packets.write().await;

        inflight_packets
            .retain(|_, p| {
                !p.is_seq_less_equal(ack_number) || {
                    let (r, rv) = Self::update_rtt(p, ack_received_at, rtt, rtt_var);
                    rtt = r;
                    rtt_var = rv;
                    size += p.size();
                    false
                }
            });

        self.rtt = rtt;
        self.rtt_var = rtt_var;

        self.state.atomic.in_flight.fetch_sub(size as u32, Ordering::AcqRel);
        size
    }

    async fn ack_one_packet(&mut self, ack_number: SequenceNumber, ack_received_at: Timestamp) -> usize {
        let mut inflight_packets = self.state.inflight_packets.write().await;
        let mut size = 0;

        if let Some(ref packet) = inflight_packets.remove(&ack_number) {
            let (r, rv) = Self::update_rtt(packet, ack_received_at, self.rtt, self.rtt_var);
            self.rtt = r;
            self.rtt_var = rv;
            size = packet.size();
            self.state.atomic.in_flight.fetch_sub(size as u32, Ordering::AcqRel);
        }

        size
    }

    async fn mark_packets_as_lost(&mut self) {
        let mut inflight_packets = self.state.inflight_packets.write().await;
        for seq in &self.tmp_packet_losts {
            if let Some(packet) = inflight_packets.get_mut(seq) {
                packet.lost = true;
            };
        }
        self.tmp_packet_losts.clear();
    }

    fn apply_congestion_control(&mut self, bytes_newly_acked: usize, in_flight: usize) {
        let lowest_relative = self.delay_history.lowest_relative();

        let cwnd = self.state.cwnd() as usize;
        // let before = cwnd;

        let cwnd_reached = in_flight + bytes_newly_acked + self.packet_size() > cwnd;

        if cwnd_reached {
            let cwnd = if self.slow_start {
                (cwnd + bytes_newly_acked) as u32
            } else {
                let window_factor = I48F16::from_num(bytes_newly_acked as i64) / in_flight as i64;
                let delay_factor = I48F16::from_num(TARGET - lowest_relative.as_i64()) / TARGET;

                let gain = (window_factor * delay_factor) * 3000;

                (cwnd as i32 + gain.to_num::<i32>()).max(0) as u32
            };
            let cwnd = self.state.remote_window().min(cwnd);
            self.state.set_cwnd(cwnd);

            //println!("CWND: {:?} BEFORE={}", cwnd, before);

            //println!("!! CWND CHANGED !! {:?} WIN_FACT {:?} DELAY_FACT {:?} GAIN {:?}",
            // cwnd, window_factor, delay_factor, gain);
        }
    }

    fn packet_size(&self) -> usize {
        let is_ipv4 = self.addr.is_ipv4();

        // TODO: Change this when MTU discovery is implemented
        if is_ipv4 {
            UDP_IPV4_MTU - HEADER_SIZE
        } else {
            UDP_IPV6_MTU - HEADER_SIZE
        }
    }
}