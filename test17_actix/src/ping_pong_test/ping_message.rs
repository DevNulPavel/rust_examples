use actix::{
    prelude::{
        *
    }
};
use super::{
    ping_response::{
        PingResponse
    }
};

// Define message
pub struct Ping{
}

impl Message for Ping {
    /// Тип значения, в которое данное сообщение будет возвращаться актором, которое обрабатывает Message
    type Result = Option<PingResponse>;
}