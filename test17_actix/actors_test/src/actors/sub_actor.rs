use crate::value_message::ValuesMessage;
use crate::calc_result::CalcResult;

// Описание нашего актора
#[derive(Default)]
pub struct SubActor{
    messages_processed: u32 // Контекст актора
}

// Реализация трейта актора
impl actix::Actor for SubActor {
    // Описываем мнотопоточый контекст актора
    type Context = actix::SyncContext<Self>;
}

// Описываем обработку сообщения SumMessage для нашего актора
impl actix::Handler<ValuesMessage> for SubActor {
    //type Result = Option<SumResult>;   // Описываем возвращаемое значение для актора
    type Result = CalcResult;   // Описываем возвращаемое значение для актора, реализовали MessageResponse

    // Обработчик поступившего сообщения для актора
    fn handle(&mut self, msg: ValuesMessage, _ctx: &mut Self::Context) -> Self::Result {
        // Обработка происходит из только одного потока, синхронизация не нужна
        self.messages_processed += 1;

        // Вычисляем
        let sub_value: i32 = msg.x - msg.y;
        
        // Результат
        Self::Result::new(sub_value, self.messages_processed)
    }
}
