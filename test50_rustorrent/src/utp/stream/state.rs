use std::{
    sync::{
        atomic::{
            Ordering,
            AtomicU8
        }
    }
};
use async_std::{
    sync::{
        RwLock
    }
};
use shared_arena::{
    ArenaBox
};
use crate::{
    cache_line::{
        CacheAligned
    },
    utils::{
        Map
    },
    utp::{
        sequence_number::{
            SequenceNumber
        },
        packet::{
            Packet
        },
        state::{
            State as UtpState
        },
        connection_id::{
            ConnectionId
        }
    }
};
use super::{
    atomic_state::{
        AtomicState
    }
};

#[derive(Debug)]
pub struct State {
    // Атомарное состояние
    pub atomic: AtomicState,

    /// Отправленные пакеты, но не полученные подтверждения ACK для них
    /// Map с синхронизацией, 
    ///     где ключ - это номер отправленного пакета
    ///     значение - бокс над пакетом 
    // TODO: Зачем Arena box??
    pub inflight_packets: RwLock<Map<SequenceNumber, ArenaBox<Packet>>>,
}

impl State {
    /// Добавить пакет к списку неподтвержденных
    pub async fn add_packet_inflight(&self, seq_num: SequenceNumber, packet: ArenaBox<Packet>) {
        // Размера нашего пакета
        let size = packet.size();

        // Получаем блокировку на запись
        let mut inflight_packets = self.inflight_packets.write().await;
        
        // Добавляем пакет под номером последовательности
        inflight_packets.insert(seq_num, packet);

        // Добавляем размер пакета к общему количество неподтвержденных данных
        self.atomic.in_flight.fetch_add(size as u32, Ordering::AcqRel);
    }

    /// Удаляем все пакеты до данного номера
    /// Возвращаем размер данных оставшихся пакетов
    pub async fn remove_packets(&self, ack_number: SequenceNumber) -> usize {
        let mut size = 0;

        // Получаем блокировку
        let mut inflight_packets = self.inflight_packets.write().await;

        // Сохраняем только нужные нам пакеты в мапе
        inflight_packets
            .retain(|_, p| {
                // Если номер у пакета не меньше или равен номеру последовательности - удаляем
                // Либо не удаляем + увеличиваем размер данных активных пакетов
                !p.is_seq_less_equal(ack_number) || (false, size += p.size()).0
            });

        // Обновляем размер данных неподтвержденных пакетов
        self.atomic.in_flight.fetch_sub(size as u32, Ordering::AcqRel);

        size
    }

    /// Удаляем пакет с конкретным номером
    /// Возвращаем размер удаленного пакета
    async fn remove_packet(&self, ack_number: SequenceNumber) -> usize {
        // Получаем блокировку
        let mut inflight_packets = self.inflight_packets.write().await;

        // Удаляем пакет + получаем размер этого пакета
        let size = inflight_packets.remove(&ack_number)
                                   .map(|p| p.size())
                                   .unwrap_or(0);

        // Отнимаем размер у объема активных данных
        self.atomic.in_flight.fetch_sub(size as u32, Ordering::AcqRel);

        size
    }

    /// Объем данных неподтвержденных пакетов
    pub fn inflight_size(&self) -> usize {
        self.atomic.in_flight.load(Ordering::Acquire) as usize
    }

    pub fn utp_state(&self) -> UtpState {
        self.atomic.utp_state.load(Ordering::Acquire).into()
    }
    pub fn set_utp_state(&self, state: UtpState) {
        self.atomic.utp_state.store(state.into(), Ordering::Release)
    }

    pub fn recv_id(&self) -> ConnectionId {
        self.atomic.recv_id.load(Ordering::Relaxed).into()
    }
    pub fn set_recv_id(&self, recv_id: ConnectionId) {
        self.atomic.recv_id.store(recv_id.into(), Ordering::Release)
    }

    pub fn send_id(&self) -> ConnectionId {
        self.atomic.send_id.load(Ordering::Relaxed).into()
    }
    pub fn set_send_id(&self, send_id: ConnectionId) {
        self.atomic.send_id.store(send_id.into(), Ordering::Release)
    }

    pub fn ack_number(&self) -> SequenceNumber {
        self.atomic.ack_number.load(Ordering::Acquire).into()
    }
    pub fn set_ack_number(&self, ack_number: SequenceNumber) {
        self.atomic.ack_number.store(ack_number.into(), Ordering::Release)
    }

    pub fn seq_number(&self) -> SequenceNumber {
        self.atomic.seq_number.load(Ordering::Acquire).into()
    }
    pub fn set_seq_number(&self, seq_number: SequenceNumber) {
        self.atomic.seq_number.store(seq_number.into(), Ordering::Release)
    }
    /// Increment seq_number and returns its previous value
    pub fn increment_seq(&self) -> SequenceNumber {
        self.atomic.seq_number.fetch_add(1, Ordering::AcqRel).into()
    }

    pub fn remote_window(&self) -> u32 {
        self.atomic.remote_window.load(Ordering::Acquire)
    }
    pub fn set_remote_window(&self, remote_window: u32) {
        self.atomic.remote_window.store(remote_window, Ordering::Release)
    }

    pub fn cwnd(&self) -> u32 {
        self.atomic.cwnd.load(Ordering::Acquire)
    }

    pub fn set_cwnd(&self, cwnd: u32) {
        self.atomic.cwnd.store(cwnd, Ordering::Release)
    }
}

impl State {
    pub fn with_utp_state(utp_state: UtpState) -> State {
        State {
            atomic: AtomicState {
                utp_state: CacheAligned::new(AtomicU8::new(utp_state.into())),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl Default for State {
    fn default() -> State {
        State {
            atomic: Default::default(),

            // delay: Delay::default(),
            // current_delays: VecDeque::with_capacity(16),
            // last_rollover: Instant::now(),
            // congestion_timeout: Duration::from_secs(1),
            // flight_size: 0,
            // srtt: 0,
            // rttvar: 0,

            inflight_packets: Default::default(),
            // delay_history: DelayHistory::new(),
            // ack_duplicate: 0,
            // last_ack: SequenceNumber::zero(),
            // lost_packets: VecDeque::with_capacity(100),
            // nlost: 0,
        }
    }
}