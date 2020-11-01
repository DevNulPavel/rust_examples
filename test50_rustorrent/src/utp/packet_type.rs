use std::{
    convert::{
        TryFrom, 
        TryInto
    }
};
use super::{
    error::{
        UtpError
    },
    Result
};


#[derive(Debug, PartialEq, Eq)]
pub enum PacketType {
    /// Обычный пакет данных. Сокет в подключенном состоянии и имеет данные для
    /// отправки. ST_DATA пакет всегда имеет данные.
    Data,

    /// Завершение соединения. Это последний пакет. Он закрывает соединение.
    /// Похоже на TCP FIN флаг. Данное соединение никогда не будет иметь 
    /// номер последовательности больше, чем номер данного пакета.
    /// Сокет записывает номер последовательности как eof_pkt.
    /// Это позволяет сокету ожидать пакета, который может быть все еще пропавшим
    /// и доставлять вне очереди даже после получения ST_FIN пакета.
    Fin,

    /// Пакет состояния. Используется для передачи ACK без данных. 
    /// Пакеты, которые не включают любую нагрузку, не увеличивают seq_nr
    State,

    /// Заканчиваем соединение принудительно. Похоже на TCP RST флаг.
    /// Удаленный хост не имеет какого-то состояния для данного соединения.
    Reset,

    /// Connect SYN. Similar to TCP SYN flag, this packet initiates a
    /// connection. The sequence number is initialized to 1. The connection
    /// ID is initialized to a random number. The syn packet is special,
    /// all subsequent packets sent on this connection (except for re-sends
    /// of the ST_SYN) are sent with the connection ID + 1. The connection
    /// ID is what the other end is expected to use in its responses.
    /// When receiving an ST_SYN, the new socket should be initialized with
    /// the ID in the packet header. The send ID for the socket should
    /// be initialized to the ID + 1. The sequence number for the return
    /// channel is initialized to a random number. The other end expects an
    /// ST_STATE packet (only an ACK) in response.
    Syn,
}

impl TryFrom<u8> for PacketType {
    type Error = UtpError;

    fn try_from(type_version: u8) -> Result<PacketType> {
        let packet_type = type_version >> 4;
        match packet_type {
            0 => Ok(PacketType::Data),
            1 => Ok(PacketType::Fin),
            2 => Ok(PacketType::State),
            3 => Ok(PacketType::Reset),
            4 => Ok(PacketType::Syn),
            _ => Err(UtpError::UnknownPacketType),
        }
    }
}

impl Into<u8> for PacketType {
    fn into(self) -> u8 {
        match self {
            PacketType::Data => 0,
            PacketType::Fin => 1,
            PacketType::State => 2,
            PacketType::Reset => 3,
            PacketType::Syn => 4,
        }
    }
}