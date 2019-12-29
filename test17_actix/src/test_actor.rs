/*extern crate futures;

use actix::prelude::*;
use actix::*;
use futures::{future, Future};

struct MyActor{
    value: i32
}

impl Actor for MyActor {
    // Контекстом актора будет непосредственно сам актор
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
       println!("I am alive!");
       self.value += 10;
       System::current().stop(); // <- stop system
    }
}

pub fn test_actor_create(){
    let system = System::new("test");

    let actor = MyActor{ value: 10 };
    let addr = actor.start();

    system.run().unwrap();
}
*/