use actix::prelude::*;
use crate::value_message::ValuesMessage;
use crate::calc_result::CalcResult;


// Описание нашего актора
#[derive(Default)]
pub struct SummatorActor{
    messages_processed: u32 // Контекст актора
}

// Реализация трейта актора
impl actix::Actor for SummatorActor {
    // Описываем однопоточный контекст актора
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Можно ограничить размер очереди
        // https://actix.rs/book/actix/sec-4-context.html
        ctx.set_mailbox_capacity(1);
        println!("Sum actor started, state: {:?}", ctx.state());
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        println!("Sum actor stopped, state: {:?}", ctx.state());
    }
}

// Описываем обработку сообщения SumMessage для нашего актора
impl actix::Handler<ValuesMessage> for SummatorActor {
    //type Result = Option<SumResult>;   // Описываем возвращаемое значение для актора
    type Result = CalcResult;   // Описываем возвращаемое значение для актора, реализовали MessageResponse

    // Обработчик поступившего сообщения для актора
    fn handle(&mut self, msg: ValuesMessage, _ctx: &mut Self::Context) -> Self::Result {
        /*if _ctx.connected(){
            println!("Context is connected");
        }
        println!("Actor state: {:?}", _ctx.state());*/

        // Обработка происходит из только одного потока, синхронизация не нужна
        self.messages_processed += 1;

        // Вычисляем
        let sum_value: i32 = msg.x + msg.y;
        
        // Результат
        Self::Result::new(sum_value, self.messages_processed)
    }
}