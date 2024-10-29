use hyper::rt::Executor;
use std::future::Future;

#[derive(Clone)]
pub struct AsynkExecutor;

impl<Fut> Executor<Fut> for AsynkExecutor
where
    Fut: Future + Send + 'static,
    Fut::Output: Send + 'static,
{
    fn execute(&self, fut: Fut) {
        asynk::spawn(fut);
    }
}
