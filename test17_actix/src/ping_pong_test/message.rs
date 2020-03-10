use actix::prelude::*;
//use futures::prelude::*;
use std::io;

// Define message
pub struct Ping{
}

impl Message for Ping {
    type Result = Result<bool, io::Error>;
}