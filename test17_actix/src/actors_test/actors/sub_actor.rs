use actix::{
    prelude::{
        *
    }
};
use crate::{
    actors_test::{
        value_message::{
            ValuesMessage
        },
        calc_result::{
            CalcResult
        }
    }
};

// Описание нашего актора
#[derive(Default)]
pub struct SubActor{
    messages_processed: u32 // Контекст актора
}

// Реализация трейта актора
impl Actor for SubActor {
    // Описываем мнотопоточый контекст актора
    type Context = SyncContext<Self>;
}

// Описываем обработку сообщения SumMessage для нашего актора
impl Handler<ValuesMessage> for SubActor {
    type Result = Result<CalcResult, ()>;   // Описываем возвращаемое значение для актора, реализовали MessageResponse

    // Обработчик поступившего сообщения для актора
    fn handle(&mut self, msg: ValuesMessage, _ctx: &mut Self::Context) -> Self::Result {
        // Обработка происходит из только одного потока, синхронизация не нужна
        self.messages_processed += 1;

        // Вычисляем
        let sub_value: i32 = msg.x - msg.y;
        
        // Результат
        Ok(CalcResult::new(sub_value, self.messages_processed))
    }
}
