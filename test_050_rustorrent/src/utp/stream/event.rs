use shared_arena::{
    ArenaBox
};
use crate::{
    utp::{
        packet::{
            Packet
        }
    }
};

pub enum UtpEvent {
    /// Входной пакет
    IncomingPacket {
        packet: ArenaBox<Packet>
    },
    
    // IncomingBytes {
    //     buffer: Vec<u8>,
    //     timestamp: Timestamp
    // },

    /// Тик
    Tick
}