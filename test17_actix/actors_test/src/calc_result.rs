
#[derive(Default, Debug)]
pub struct CalcResult{
    pub result: i32,
    pub operations_count: u32
}

impl CalcResult{
    pub fn new(result: i32, operations_count: u32)-> CalcResult{
        CalcResult{ result, operations_count }
    }
}

// Для того, чтобы не использовать Option или Result в описании типа Result у сообщения
// и у актора - мы реализуем данный трейт для результата сообщения
// Для Option/Result - уже реализовано в самой библиотеке
impl<A, M> actix::dev::MessageResponse<A, M> for CalcResult
where
    A: actix::Actor,
    M: actix::Message<Result = CalcResult>
{
    fn handle<R: actix::dev::ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
        if let Some(tx) = tx {
            tx.send(self);
        }
    }
}
