extern crate futures;

use actix::prelude::*;
use actix::*;
use futures::{future, Future};


// Содержимое нашего сообщения
#[derive(Default)]
struct SumMessage(usize, usize);

// Реализация трейта Message для нашего сообщения
impl Message for SumMessage {
    // описываем тип возвращаемого значения на сообщение
    type Result = Result<(usize, i32), String>;
}

#[derive(Default)]
struct SumResult(usize, i32);

/////////////////////////////////////////////

// Описание нашего актора
#[derive(Default)]
struct SummatorActor{
    messages_processed: i32 // Контекст актора
}

// Реализация трейта актора
impl Actor for SummatorActor {
    type Context = Context<Self>;
}

// Описываем обработку сообщения SumMessage для нашего актора
impl Handler<SumMessage> for SummatorActor {
    type Result = Result<(usize, i32), String>;   // Описываем возвращаемое значение для актора

    // Обработчик поступившего сообщения для актора
    fn handle(&mut self, msg: SumMessage, ctx: &mut Context<Self>) -> Self::Result {
        self.messages_processed += 1;

        let sum_value: usize = msg.0 + msg.1;
        
        Ok((sum_value, self.messages_processed))
    }
}

/////////////////////////////////////////////

pub fn test_actor_messages() {
    let sys = System::new("test");

    // Создаем нашего актора
    let actor = SummatorActor::default();

    // Закидываем его в систему c получением канала отправки сообщений
    let addr: Addr<SummatorActor> = actor.start();

    // Отбправляем сообщение, получаем объект с отложенным результатом
    let res = addr.send(SumMessage(10, 5));

    Arbiter::spawn(res.map(|res| {
        match res {
            Ok(result) => println!("SUM: {}", result),
            _ => println!("Something wrong"),
        }

        System::current().stop();
        future::result(Ok(()))
    }));

    sys.run();
}