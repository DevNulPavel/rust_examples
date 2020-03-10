use super::message::Ping;
use super::actor::MyActor;

use actix::prelude::*;
use futures::prelude::*;

pub fn test_ping_pong() {
    let sys = System::new("example");

    // Создаем нашего актора, такой спооб нужен для быстрого создания и запуска потом
    let sum_actor = MyActor{};

    let addr = sum_actor.start();

    // Send Ping message.
    // send() message returns Future object, that resolves to message result
    let result = addr.send(Ping{});

    // result.then(|res|{
    //     println!("{:?}", res);
    // });

    // spawn future to reactor
    let new_fut = result
        .map(|res| {
            match res {
                Ok(result) => println!("Got result: {:?}", result),
                Err(err) => println!("Got error: {}", err),
            }
            actix::System::current().stop();
            ()
        });
    Arbiter::spawn(new_fut);

    sys.run().unwrap();
}