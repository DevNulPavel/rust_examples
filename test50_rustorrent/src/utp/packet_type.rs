use std::{
    convert::{
        TryFrom
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

    /// SYN флаг. Поход на TCP SYN флаг, данный пакет инициирует соединение.
    /// Номер последовательности инициализируется значением 1
    /// Connection ID инициализируется рандомным значением.
    /// SYN пакет особенный, все последующие пакеты отправленные данному соединению,
    /// кроме переотправки ST_SYN, отправляются с connection ID + 1.
    /// Connection ID - это то, что другая сторона будет использовать в своих ответах.
    /// При получении ST_SYN новый сокет должен быть инициализирован с ID в хедере пакета.
    /// Send ID для сокета должен быть инициализирован как ID + 1.
    /// Номер последовательности для return channel инициализируется рандомным значением.
    /// Другая сторона ожидает ST_STATE пакет (только ACK) в ответ.
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