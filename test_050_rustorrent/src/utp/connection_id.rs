use std::{
    ops::{
        Add
    }
};
use rand::{
    Rng
};


/// Структура заголовка
#[derive(Copy, Clone, Debug)]
pub struct ConnectionId(u16);

impl From<u16> for ConnectionId {
    fn from(byte: u16) -> ConnectionId {
        ConnectionId(byte)
    }
}

impl Into<u16> for ConnectionId {
    fn into(self) -> u16 {
        self.0
    }
}

impl Add<u16> for ConnectionId {
    type Output = Self;

    fn add(self, o: u16) -> ConnectionId {
        ConnectionId(self.0 + o)
    }
}

impl ConnectionId {
    pub fn make_ids() -> (ConnectionId, ConnectionId) {
        let id = rand::thread_rng().gen::<u16>();
        if id == 0 {
            (id.into(), (id + 1).into())
        } else {
            ((id - 1).into(), id.into())
        }
    }
}
