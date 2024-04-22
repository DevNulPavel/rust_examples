use actix::{
    prelude::{
        *
    }
};
use super::{
    counter::{
        CounterIncMessage
    }
};

// Define message
pub struct PingSubscribe{
    pub observer: Recipient<CounterIncMessage>
}

impl Message for PingSubscribe {
    /// Тип значения, в которое данное сообщение будет возвращаться актором, которое обрабатывает Message
    type Result = ();
}

impl PingSubscribe{
    pub fn new(observer: Recipient<CounterIncMessage>) -> PingSubscribe{
        PingSubscribe{
            observer
        }
    }
}