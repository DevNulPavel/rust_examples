use actix::{
    prelude::{
        *
    }
};
use super::{
    counter_message::{
        CounterIncMessage
    }
};

pub struct CounterActor{
    name: String,
    total_inreases: u32
}

impl Actor for CounterActor {
    type Context = Context<Self>;
}

impl Handler<CounterIncMessage> for CounterActor{
    type Result = ();

    /// Вызывается для обработке сообщения в потоке актора
    fn handle(&mut self, _msg: CounterIncMessage, _: &mut Context<Self>) -> Self::Result {
        self.total_inreases += 1;
        println!("Counter {}: processed count increased, new value is: {}", self.name, self.total_inreases);
    }
}

impl CounterActor{
    pub fn new(name: &str) -> CounterActor{
        CounterActor{
            name: name.to_string(),
            total_inreases: 0
        }
    }
}