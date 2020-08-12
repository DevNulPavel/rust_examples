use std::{
    time
};
/*use futures::{
    prelude::{
        *
    }
};*/
use actix::{
    prelude::{
        *
    }
};
use super::{
    ping_message::{
        Ping
    },
    ping_subscribe_message::{
        PingSubscribe
    },
    ping_response::{
        PingResponse
    },
    counter::{
        CounterIncMessage
    }
};

// Define actor
#[derive(Default)]
pub struct PingActor{
    observers: Vec<Recipient<CounterIncMessage>>
}

impl PingActor {
    pub fn new() -> PingActor{
        PingActor{
            observers: vec![]
        }
    }
}

// Непосредственно реализация нашего актора
impl Actor for PingActor {
    type Context = Context<Self>;

    /// Вызывается когда актор пулится в первый раз
    fn started(&mut self, ctx: &mut Self::Context) {
        // Можно выставить размер ящика сообщений
        ctx.set_mailbox_capacity(4);
        println!("Actor is alive");
    }

    /// Вызывается перед остановкой
    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // Останавливаем 
        Running::Stop
        // Продолжить исполнение
        //Running::Continue
    }

    /// Вызывается после остановки актора.
    /// Данный метод может быть использован для выполнения 
    /// необходимой очистки или для спавна новых акторов.
    /// Это финальное состояние, после этого актор будет уничтожен и вызван drop.
    fn stopped(&mut self, _ctx: &mut Self::Context) {
       println!("Actor stopped");
    }
}

impl Handler<PingSubscribe> for PingActor{
    type Result = ();

    /// Вызывается для обработке сообщения в потоке актора
    fn handle(&mut self, msg: PingSubscribe, _: &mut Context<Self>) -> Self::Result {
        self.observers.push(msg.observer);
    }
}

/// Обработчик сообщения Ping
impl Handler<Ping> for PingActor {
    /// Тип, который возвращается после обработки сообщения.
    /// Точно такое же значение, как у реализации Ping
    type Result = Option<PingResponse>;

    /// Вызывается для обработке сообщения в потоке актора
    fn handle(&mut self, _msg: Ping, ctx: &mut Context<Self>) -> Self::Result {
        // Оповещаем всех обзерверов об получении пинга
        self.observers
            .iter()
            .for_each(|recepient|{
                recepient.do_send(CounterIncMessage{}).unwrap();
            });

        println!("Ping received");

        let _handle = ctx.run_later(time::Duration::from_secs(3), |_actor: &mut Self, ctx: &mut Context<Self>|{
            println!("Ping received delayed");
            // Останавливаем актора
            ctx.stop(); 

            // Останавливаем всю систему
            System::current().stop();
        });
        //let _request: Request<PingPongActor, Ping> = _ctx.address().send(Ping{});
        //let request_future: &Future<Output=_> = &request;
        //(request as &Future<Output=_>)

        Some(PingResponse{})
    }
}