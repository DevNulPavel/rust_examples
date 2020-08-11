use std::{
    io,
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
    message::{
        Ping
    }
};

// Define actor
#[derive(Default)]
pub struct PingPongActor{
}

// Непосредственно реализация нашего актора
impl Actor for PingPongActor {
    type Context = Context<Self>;

    /// Вызывается когда актор пулится в первый раз
    fn started(&mut self, _ctx: &mut Self::Context) {
       println!("Actor is alive");
    }

    /// Вызывается после остановки актора.
    /// Данный метод может быть использован для выполнения 
    /// необходимой очистки или для спавна новых акторов.
    /// Это финальное состояние, после этого актор будет уничтожен и вызван drop.
    fn stopped(&mut self, _ctx: &mut Self::Context) {
       println!("Actor stopped");
    }
}

/// Обработчик сообщения Ping
impl Handler<Ping> for PingPongActor {
    /// Тип, который возвращается после обработки сообщения
    type Result = Result<bool, io::Error>;

    /// Вызывается для обработке сообщения в потоке актора
    fn handle(&mut self, _msg: Ping, ctx: &mut Context<Self>) -> Self::Result {
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

        Ok(true)
    }
}