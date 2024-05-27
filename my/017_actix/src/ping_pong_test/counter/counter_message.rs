use actix::{
    prelude::{
        *
    }
};

pub struct CounterIncMessage{
}

impl Message for CounterIncMessage {
    type Result = ();
}