use std::{
    io
};
use actix::{
    prelude::{
        *
    }
};

// Define message
pub struct Ping{
}

impl Message for Ping {
    /// Тип значения, в которое данное сообщение будет резолвиться?
    type Result = Result<bool, io::Error>;
}