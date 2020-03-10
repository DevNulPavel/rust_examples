use actix::prelude::*;
//use futures::prelude::*;
use std::io;
use super::message::Ping;

// Define actor
#[derive(Default)]
pub struct MyActor{
}

// Provide Actor implementation for our actor
impl Actor for MyActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
       println!("Actor is alive");
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
       println!("Actor is stopped");
    }
}

/// Define handler for `Ping` message
impl Handler<Ping> for MyActor {
    type Result = Result<bool, io::Error>;

    fn handle(&mut self, _msg: Ping, _ctx: &mut Context<Self>) -> Self::Result {
        println!("Ping received");

        Ok(true)
    }
}