use std::{
    convert::{
        TryInto
    }
};
use super::{
    packet_type::{
        PacketType
    },
    error::{
        UtpError
    },
    timestamp::{
        Timestamp
    },
    sequence_number::{
        SequenceNumber
    },
    delay::{
        Delay
    },
    ExtensionType,
    ConnectionId,
    Result
};

/// Формат заголовка пакета
///
/// 0       4       8               16              24              32
/// +-------+-------+---------------+---------------+---------------+
/// | type  | ver   | extension     | connection_id                 |
/// +-------+-------+---------------+---------------+---------------+
/// | timestamp_microseconds                                        |
/// +---------------+---------------+---------------+---------------+
/// | timestamp_difference_microseconds                             |
/// +---------------+---------------+---------------+---------------+
/// | wnd_size                                                      |
/// +---------------+---------------+---------------+---------------+
/// | seq_nr                        | ack_nr                        |
/// +---------------+---------------+---------------+---------------+

pub(super) const HEADER_SIZE: usize = std::mem::size_of::<Header>();

// Структура, имеющая Сшное представление, данные в структуре упаковываются плотно без пробелов
#[repr(C, packed)]
pub struct Header {
    /// Версия протокола + тип пакета
    ///     type = type_version >> 4
    ///     version = type_version & 0x0F
    /// Текущая версия 1
    /// Поле описывает тип пакетов
    type_version: u8,

    /// Тип первого расширения в связанно списке заголовков расширений
    /// 0 значит отсутствие расширений
    extension: u8,

    /// Это случайное и уникальное значение, которое идентифицирует все пакеты, 
    /// которые принадлежат данному соединению.
    /// Каждый сокет имеет одинаковое значение ID для отправки пакетов,
    /// но другое значение для получения пакетов.
    /// Конечная точка, инициализирующая соединение, решает какой ID использовать
    /// а возвращаемое имеет тот же ID + 1
    connection_id: u16,

    /// Поле в микросекундах, когда данный пакет был отправлен.
    /// Устанавливается используя gettimeofday() из posix или QueryPerformanceTimer()
    /// на винде.
    /// Наибольшее разрешение данного таймера лучше.
    /// Ближайшее к актуальному transmit time установлено, тем лучше.
    timestamp_micro: u32,

    /// This is the difference between the local time and the timestamp in the last
    /// received packet, at the time the last packet was received. This is the
    /// latest one-way delay measurement of the link from the remote peer to the local
    /// machine. When a socket is newly opened and doesn't have any delay
    /// samples yet, this must be set to 0.
    timestamp_difference_micro: u32,
    /// Advertised receive window. This is 32 bits wide and specified in bytes.
    /// The window size is the number of bytes currently in-flight, i.e. sent but
    /// not acked. The advertised receive window lets the other end cap the
    /// window size if it cannot receive any faster, if its receive buffer is
    /// filling up.
    /// When sending packets, this should be set to the number of bytes left in
    /// the socket's receive buffer.
    window_size: u32,
    /// This is the sequence number of this packet. As opposed to TCP, uTP
    /// sequence numbers are not referring to bytes, but packets. The sequence
    /// number tells the other end in which order packets should be served back
    /// to the application layer.
    seq_nr: u16,
    /// This is the sequence number the sender of the packet last received in the
    /// other direction.
    ack_nr: u16,
}

impl Header {
    pub(super) fn get_type(&self) -> Result<PacketType> {
        self.check_version()?;
        self.type_version.try_into()
    }

    pub(super) fn check_version(&self) -> Result<()> {
        match self.type_version & 0x0F {
            1 => Ok(()),
            _ => Err(UtpError::WrongVersion)
        }
    }

    // Getters
    pub(super) fn get_connection_id(&self) -> ConnectionId {
        u16::from_be(self.connection_id).into()
    }
    pub(super) fn get_version(&self) -> u8 {
        self.type_version & 0xF
    }
    pub(super) fn get_timestamp(&self) -> Timestamp {
        u32::from_be(self.timestamp_micro).into()
    }
    pub(super) fn get_timestamp_diff(&self) -> Delay {
        u32::from_be(self.timestamp_difference_micro).into()
    }
    pub(super) fn get_window_size(&self) -> u32 {
        u32::from_be(self.window_size)
    }
    pub(super) fn get_seq_number(&self) -> SequenceNumber {
        SequenceNumber::from_be(self.seq_nr)
    }
    pub(super) fn get_ack_number(&self) -> SequenceNumber {
        SequenceNumber::from_be(self.ack_nr)
    }
    pub(super) fn get_extension_type(&self) -> ExtensionType {
        ExtensionType::from(self.extension)
    }
    pub(super) fn has_extension(&self) -> bool {
        self.extension != 0
    }

    // Setters
    pub(super) fn set_connection_id(&mut self, id: ConnectionId) {
        self.connection_id = u16::to_be(id.into());
    }
    pub(super) fn set_timestamp(&mut self, timestamp: Timestamp) {
        self.timestamp_micro = u32::to_be(timestamp.into());
    }
    pub(super) fn set_timestamp_diff(&mut self, delay: Delay) {
        self.timestamp_difference_micro = u32::to_be(delay.into());
    }
    pub(super) fn set_window_size(&mut self, window_size: u32) {
        self.window_size = u32::to_be(window_size);
    }
    pub(super) fn set_seq_number(&mut self, seq_number: SequenceNumber) {
        self.seq_nr = seq_number.to_be();
    }
    pub(super) fn set_ack_number(&mut self, ack_number: SequenceNumber) {
        self.ack_nr = ack_number.to_be();
    }

    // fn update_timestamp(&mut self) {
    //     self.set_timestamp(Timestamp::now());
    // }

    pub fn new(packet_type: PacketType) -> Header {
        let packet_type: u8 = packet_type.into();
        Header {
            type_version: packet_type << 4 | 1,
            ..Default::default()
        }
    }

    /// TODO: Вроде как уже учитываются BigEndian?
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { &*(self as *const Header as *const [u8; std::mem::size_of::<Header>()]) }
    }
}

impl Default for Header {
    fn default() -> Header {
        let packet_type: u8 = PacketType::Data.into();
        Header {
            type_version: packet_type << 4 | 1,
            extension: 0,
            connection_id: 0,
            timestamp_micro: 0,
            timestamp_difference_micro: 0,
            window_size: 0,
            seq_nr: 0,
            ack_nr: 0,
        }
    }
}

impl std::fmt::Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Header")
         .field("type", &self.get_type())
         .field("version", &self.get_version())
         // .field("extension", &self.get_extension_type())
         .field("connection_id", &self.get_connection_id())
         .field("timestamp", &self.get_timestamp())
         .field("timestamp_difference", &self.get_timestamp_diff())
         .field("window_size", &self.get_window_size())
         .field("seq_number", &self.get_seq_number())
         .field("ack_number", &self.get_ack_number())
         .finish()
    }
}
