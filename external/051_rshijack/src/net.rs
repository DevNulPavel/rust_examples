use crate::errors::*;
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, NetworkInterface};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::{Ipv4Flags, MutableIpv4Packet};
use pnet::packet::ipv6::MutableIpv6Packet;
use pnet::packet::tcp::MutableTcpPacket;
use pnet::packet::MutablePacket;
use pnet::transport::TransportChannelType::Layer3;
use pnet::transport::{transport_channel, TransportReceiver, TransportSender};
pub use pnet::packet::tcp::{ipv4_checksum, ipv6_checksum, TcpFlags};
use log::Level;
use pktparse::ethernet;
use pktparse::tcp::{self, TcpHeader};
use pktparse::{ip, ipv4, ipv6};
use std::io::{self, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::sync::{Arc, Mutex};

/// Структурка, описывающая соединение
#[derive(Debug, Clone)]
pub struct Connection {
    pub src: SocketAddr,
    pub dst: SocketAddr,
    pub seq: Arc<Mutex<u32>>,
    pub ack: Arc<Mutex<u32>>,
}

impl Connection {
    #[inline]
    pub fn new(src: SocketAddr, dst: SocketAddr, seq: u32, ack: u32) -> Connection {
        Connection {
            src,
            dst,
            seq: Arc::new(Mutex::new(seq)),
            ack: Arc::new(Mutex::new(ack)),
        }
    }

    /// Увеличиваем счетчик seq
    #[inline]
    pub fn bump_seq(&self, inc: u32) {
        let mut guard = self.seq.lock().unwrap();
        *guard += inc;
    }

    /// Увеличиваем счетчик ack
    #[inline]
    pub fn set_ack(&self, ack: u32) {
        let mut guard = self.ack.lock().unwrap();
        *guard = ack;
    }

    #[inline]
    pub fn get_seq(&self) -> u32 {
        *self.seq.lock().unwrap()
    }

    #[inline]
    pub fn get_ack(&self) -> u32 {
        *self.ack.lock().unwrap()
    }

    /// Отправляем TCP пакет куда надо
    #[inline]
    pub fn sendtcp(&mut self, tx: &mut TransportSender, flags: u16, data: &[u8]) -> Result<()> {
        // Отправляем данные
        sendtcp(
            tx,
            &self.src,
            &self.dst,
            flags,
            self.get_seq(),
            self.get_ack(),
            data,
        )?;
        // Увеличиваем seq счетчик на длину данных
        self.bump_seq(data.len() as u32);
        Ok(())
    }

    /// Отправляем подтверждение объема данных в исходный канал связи
    #[inline]
    pub fn ack(&mut self, tx: &mut TransportSender, mut ack: u32, data: &[u8]) -> Result<()> {
        // Обновляем новое значение ACK на длину данных
        ack += data.len() as u32;
        self.set_ack(ack);

        // Отправляем ACK сообщение
        sendtcp(
            tx,
            &self.src,
            &self.dst,
            TcpFlags::ACK,
            self.get_seq(),
            ack,
            &[],
        )
    }

    /// Делаем отправку сброса
    #[inline]
    pub fn reset(&mut self, tx: &mut TransportSender) -> Result<()> {
        sendtcp(
            tx,
            &self.src,
            &self.dst,
            TcpFlags::RST,
            self.get_seq(),
            0,
            &[],
        )
    }
}

pub struct IpHeader {
    source_addr: IpAddr,
    dest_addr: IpAddr,
}

/// Получаем соединение после SEQ/ACK
#[inline]
pub fn getseqack(interface: &str, src: &SocketAddr, dst: &SocketAddr) -> Result<Connection> {
    // Запускаем отслеживание трафика
    sniff(
        interface,
        Level::Debug,
        src,
        dst,
        // Функтор, анализирующий пакет
        |ip_hdr, tcp_hdr, remaining| {
            // Пропускаем пакеты, которые имеют совершенно не те порты
            if (src.port() != tcp_hdr.source_port && src.port() != 0)
                || (dst.port() != tcp_hdr.dest_port && dst.port() != 0)
            {
                return Ok(None);
            }

            // Пропускаем пакет, если это не ACK флаг
            if !tcp_hdr.flag_ack {
                return Ok(None);
            }

            // Это нужный нам пакет - стартуем его
            Ok(Some(Connection::new(
                SocketAddr::new(ip_hdr.source_addr, tcp_hdr.source_port),
                SocketAddr::new(ip_hdr.dest_addr, tcp_hdr.dest_port),
                tcp_hdr.sequence_no + remaining.len() as u32,
                tcp_hdr.ack_no,
            )))
        },
    )
}

/// Получаем определенные данные из интерфейса
#[inline]
pub fn recv(
    tx: &mut TransportSender,
    interface: &str,
    connection: &mut Connection,
    src: &SocketAddr,
    dst: &SocketAddr,
) -> Result<()> {
    let mut stdout = io::stdout();

    // Анализируем трафик на интерейсе определенном
    sniff(
        interface,
        Level::Trace,
        src, // Исходный адрес
        dst, // Целевой адрес
        // Обработчик каждого конкретного пакета
        |_ip_hdr, tcp_hdr, remaining| {
            // Пропускаем пакеты если наши порты вообще не совпадают
            if src.port() != tcp_hdr.source_port || dst.port() != tcp_hdr.dest_port {
                return Ok(None);
            }

            // Пропускаем если флаг PSH вообще не проставлен
            if !tcp_hdr.flag_psh {
                return Ok(None);
            }

            // Если это пакет, у которого значение подтверждения (ACK) меньше
            // чем значение в заголовоке + длина данных
            if connection.get_ack() >= tcp_hdr.sequence_no + remaining.len() as u32 {
                return Ok(None);
            }

            // Пишем данные о пакете в stdout
            stdout.write_all(remaining)?;
            stdout.flush()?;

            // Отправляем подтверждение пакет
            connection.ack(tx, tcp_hdr.sequence_no, remaining)?;

            Ok(None)
        },
    )
}

/// Совпадает ли адрес с переданным
fn ipv4_addr_match(filter: &Ipv4Addr, actual: &Ipv4Addr) -> bool {
    if filter == &Ipv4Addr::UNSPECIFIED {
        true
    } else {
        filter == actual
    }
}

/// Совпадает ли адрес с переданным
fn ipv6_addr_match(filter: &Ipv6Addr, actual: &Ipv6Addr) -> bool {
    if filter == &Ipv6Addr::UNSPECIFIED {
        true
    } else {
        filter == actual
    }
}

/// Анализируем пакеты на определенном интерфейсе
pub fn sniff<F, T>(
    interface: &str,
    log_level: Level,
    src: &SocketAddr,
    dst: &SocketAddr,
    mut callback: F,
) -> Result<T>
where
    F: FnMut(IpHeader, TcpHeader, &[u8]) -> Result<Option<T>>,
{
    // Получаем список интерфейсов
    let interfaces = datalink::interfaces();
    // Находим нужный нам интерфейс
    let interface = interfaces
        .into_iter()
        .find(|iface: &NetworkInterface| iface.name == interface)
        .context("Interface not found")?;

    // Получаем канал обработки определенного интерфейса
    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => bail!("Unhandled channel type"),
        Err(e) => bail!(
            "An error occurred when creating the datalink channel: {}",
            e
        ),
    };

    // Обрабатываем прилетающие пакеты
    while let Ok(packet) = rx.next() {
        trace!("received {:?}", packet);

        // Парсим наш прилетевший пакет, получаем оставшиеся
        // наши данные + данные о пакете
        if let Ok((remaining, eth_frame)) = ethernet::parse_ethernet_frame(packet) {
            log!(log_level, "eth: {:?}", eth_frame);

            // Смотрим что у нас за пакет
            match (eth_frame.ethertype, src, dst) {
                (ethernet::EtherType::IPv4, SocketAddr::V4(src), SocketAddr::V4(dst)) => {
                    // Заголовок IPv4 пакета + оставшиеся данные пакета
                    if let Ok((remaining, ip_hdr)) = ipv4::parse_ipv4_header(remaining) {
                        log!(log_level, "ip4: {:?}", ip_hdr);

                        // Пропускаем если IP адреса совершенно не те, что нам нужны
                        if !ipv4_addr_match(src.ip(), &ip_hdr.source_addr)
                            || !ipv4_addr_match(dst.ip(), &ip_hdr.dest_addr)
                        {
                            continue;
                        }

                        // Анализируем только TCP протокол
                        if ip_hdr.protocol == ip::IPProtocol::TCP {
                            // Получаем TCP заголовок + оставшиеся данные
                            if let Ok((remaining, tcp_hdr)) = tcp::parse_tcp_header(remaining) {
                                log!(log_level, "tcp: {:?}", tcp_hdr);
                                // Формируем информацию в виде заголовка
                                let ip_hdr = IpHeader {
                                    source_addr: IpAddr::V4(ip_hdr.source_addr),
                                    dest_addr: IpAddr::V4(ip_hdr.dest_addr),
                                };
                                // Вызываем коллбек с оставшимися даннными и заголовками, которые распарсили
                                // Если нашли что-то, прекращаем обработку пакетов
                                if let Some(result) = callback(ip_hdr, tcp_hdr, remaining)? {
                                    return Ok(result);
                                }
                            }
                        }
                    }
                }
                (ethernet::EtherType::IPv6, SocketAddr::V6(src), SocketAddr::V6(dst)) => {
                    if let Ok((remaining, ip_hdr)) = ipv6::parse_ipv6_header(remaining) {
                        log!(log_level, "ip4: {:?}", ip_hdr);

                        // skip packet if src/dst ip doesn't match
                        if !ipv6_addr_match(src.ip(), &ip_hdr.source_addr)
                            || !ipv6_addr_match(dst.ip(), &ip_hdr.dest_addr)
                        {
                            continue;
                        }

                        if ip_hdr.next_header == ip::IPProtocol::TCP {
                            if let Ok((remaining, tcp_hdr)) = tcp::parse_tcp_header(remaining) {
                                log!(log_level, "tcp: {:?}", tcp_hdr);

                                let ip_hdr = IpHeader {
                                    source_addr: IpAddr::V6(ip_hdr.source_addr),
                                    dest_addr: IpAddr::V6(ip_hdr.dest_addr),
                                };
                                if let Some(result) = callback(ip_hdr, tcp_hdr, remaining)? {
                                    return Ok(result);
                                }
                            }
                        }
                    }
                }
                _ => (),
            }
        }
    }

    bail!("Reading from interface failed!")
}

/// Создаем сокет с размером буффера 4096 байт
pub fn create_socket() -> Result<(TransportSender, TransportReceiver)> {
    let protocol = Layer3(IpNextHeaderProtocols::Tcp);
    let (tx, rx) = transport_channel(4096, protocol)?;
    Ok((tx, rx))
}

/// Выполняем отправку TCP данных
pub fn sendtcp(
    tx: &mut TransportSender,
    src: &SocketAddr,
    dst: &SocketAddr,
    flags: u16,
    seq: u32,
    ack: u32,
    data: &[u8],
) -> Result<()> {
    match (src, dst) {
        (SocketAddr::V4(src), SocketAddr::V4(dst)) => {
            sendtcpv4(tx, src, dst, flags, seq, ack, data)
        }
        (SocketAddr::V6(src), SocketAddr::V6(dst)) => {
            sendtcpv6(tx, src, dst, flags, seq, ack, data)
        }
        _ => bail!("Invalid ipv4/ipv6 combination"),
    }
}

/// Отправка V4 пакетов
pub fn sendtcpv4(
    tx: &mut TransportSender,
    src: &SocketAddrV4,
    dst: &SocketAddrV4,
    flags: u16,
    seq: u32,
    ack: u32,
    data: &[u8],
) -> Result<()> {
    // Размер TCP пакета как сумма обязательных данных + размер реалных данных
    let tcp_len = MutableTcpPacket::minimum_packet_size() + data.len();
    // Размер суммарный пакета как V4 + размер TCP
    let total_len = MutableIpv4Packet::minimum_packet_size() + tcp_len;

    // Выделяем буффер нужного размера
    // TODO: Использовать выделение на стеке
    let mut pkt_buf: Vec<u8> = vec![0; total_len];

    // populate ipv4
    // Размер заголовка IPv4 вычисляем как размер V4 пакета, деленный на 4
    let ipv4_header_len = match MutableIpv4Packet::minimum_packet_size().checked_div(4) {
        Some(l) => l as u8,
        None => bail!("Invalid header len"),
    };

    // Формируем пакет
    let mut ipv4 = MutableIpv4Packet::new(&mut pkt_buf).unwrap();
    ipv4.set_header_length(ipv4_header_len); // Размер заголовка
    ipv4.set_total_length(total_len as u16); // Полный размер данных
    ipv4.set_next_level_protocol(IpNextHeaderProtocols::Tcp); // Внутри пакета будет TCP gfrtn
    ipv4.set_source(src.ip().to_owned()); // C какого адреса пакет
    ipv4.set_version(4);                  // Версия 4
    ipv4.set_ttl(64);                     // Проставляем TTL пакета в 64
    ipv4.set_destination(*dst.ip());      // Адрес целевой для пакета
    ipv4.set_flags(Ipv4Flags::DontFragment); // Не фрагментируем данный пакет
    ipv4.set_options(&[]);

    // Заполняем TCP данные в пакет IPv4
    gentcp(
        ipv4.payload_mut(),
        &SocketAddr::V4(*src),
        &SocketAddr::V4(*dst),
        flags,
        seq,
        ack,
        data,
    )?;

    // Отправляем данные
    match tx.send_to(ipv4, IpAddr::V4(*dst.ip())) {
        Ok(bytes) => {
            if bytes != total_len {
                bail!("short send count: {}", bytes)
            }
        }
        Err(e) => bail!("Could not send: {}", e),
    };

    Ok(())
}

pub fn sendtcpv6(
    tx: &mut TransportSender,
    src: &SocketAddrV6,
    dst: &SocketAddrV6,
    flags: u16,
    seq: u32,
    ack: u32,
    data: &[u8],
) -> Result<()> {
    let tcp_len = MutableTcpPacket::minimum_packet_size() + data.len();
    let total_len = MutableIpv6Packet::minimum_packet_size() + tcp_len;

    let mut pkt_buf: Vec<u8> = vec![0; total_len];

    // populate ipv6
    let mut ipv6 = MutableIpv6Packet::new(&mut pkt_buf).unwrap();
    ipv6.set_payload_length(tcp_len as u16);

    ipv6.set_next_header(IpNextHeaderProtocols::Tcp);
    ipv6.set_source(src.ip().to_owned());
    ipv6.set_version(6);
    ipv6.set_hop_limit(64);
    ipv6.set_destination(*dst.ip());

    // Заполняем данные TCP в IP пакет
    gentcp(
        ipv6.payload_mut(),
        &SocketAddr::V6(*src),
        &SocketAddr::V6(*dst),
        flags,
        seq,
        ack,
        data,
    )?;

    match tx.send_to(ipv6, IpAddr::V6(*dst.ip())) {
        Ok(bytes) => {
            if bytes != total_len {
                bail!("short send count: {}", bytes)
            }
        }
        Err(e) => bail!("Could not send: {}", e),
    };

    Ok(())
}

/// Заполняем IPv4 / IPv6 пакет данными о TCP пакете
fn gentcp(
    payload_mut: &mut [u8], // Данные пакета IPv4 / IPv6 без заголовка 
    src: &SocketAddr,       // Исходный адрес
    dst: &SocketAddr,       // Целевой адрес
    flags: u16,             // Разные флаги
    seq: u32,               // Номера SEQ/ACK
    ack: u32,
    data: &[u8],            // Уже непосредственно TCP данные
) -> Result<()> {
    // Размер заголовка как 1/4 размера пакета TCP
    let tcp_header_len = match MutableTcpPacket::minimum_packet_size().checked_div(4) {
        Some(l) => l as u8,
        None => bail!("Invalid header len"),
    };

    // Создаем TCP пакет поверх других данных
    // Проверяется размер пакета, возвращается Some если размер нормальный
    let mut tcp = MutableTcpPacket::new(payload_mut).unwrap();
    tcp.set_data_offset(tcp_header_len);    // Смещение данных после заголовка
    tcp.set_source(src.port());             // Исходный порт
    tcp.set_destination(dst.port());        // Целевой порт
    tcp.set_sequence(seq);                  // SEQ
    tcp.set_acknowledgement(ack);           // ACK
    tcp.set_flags(flags);                   // Флаги

    // Устанавливаем минимальный размер окна для пакетов
    let mut window = data.len() as u16;
    if window == 0 {
        window = 4;
    }
    tcp.set_window(window);

    // Собственно сами данные
    tcp.set_payload(data);

    // Выполняем расчет контрольной суммы для нашего пакеты
    let chk = match (src, dst) {
        (SocketAddr::V4(src), SocketAddr::V4(dst)) => {
            ipv4_checksum(&tcp.to_immutable(), src.ip(), dst.ip())
        }
        (SocketAddr::V6(src), SocketAddr::V6(dst)) => {
            ipv6_checksum(&tcp.to_immutable(), src.ip(), dst.ip())
        }
        _ => bail!("Invalid ipv4/ipv6 combination"),
    };

    // Затем устанавливаем контрольную сумму
    tcp.set_checksum(chk);

    Ok(())
}
