use std::io;
use actix::prelude::*;
//use actix::dev::{MessageResponse, ResponseChannel};
use futures::Future;

/// Define message
struct Ping;

impl Message for Ping {
    type Result = Result<bool, io::Error>;
}


// Define actor
struct MyActor;

// Provide Actor implementation for our actor
impl Actor for MyActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
       println!("Actor is alive");
    }

    fn stopped(&mut self, ctx: &mut Context<Self>) {
       println!("Actor is stopped");
    }
}

/// Define handler for `Ping` message
impl Handler<Ping> for MyActor {
    type Result = Result<bool, io::Error>;

    fn handle(&mut self, msg: Ping, ctx: &mut Context<Self>) -> Self::Result {
        println!("Ping received");

        Ok(true)
    }
}

pub fn test_ping_pong() {
    let sys = System::new("example");

    // Start MyActor in current thread
    let addr = MyActor.start();

    // Send Ping message.
    // send() message returns Future object, that resolves to message result
    let result = addr.send(Ping);

    // result.then(|res|{
    //     println!("{:?}", res);
    // });

    // spawn future to reactor
    Arbiter::spawn(
        result.map(|res| {
            match res {
                Ok(result) => println!("Got result: {}", result),
                Err(err) => println!("Got error: {}", err),
            }
        })
        .map_err(|e| {
            println!("Actor is probably dead: {}", e);
        }));

    sys.run();
}