use futures::{channel::oneshot, FutureExt};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use super::JoinError;

////////////////////////////////////////////////////////////////////////////////

/// Поддержка ожидания завершения работы
pub struct JoinHandle<T>(oneshot::Receiver<T>);

impl<T> JoinHandle<T>
where
    T: Send + 'static,
{
    // Создание нового ожидателя
    pub(super) fn new(rx: oneshot::Receiver<T>) -> Self {
        Self(rx)
    }
}

/// Join у нас будет работать как футура тоже
impl<T> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Здесь мы просто пробрасываем полинг в дочерний канал
        self.0.poll_unpin(cx).map_err(|_| JoinError)
    }
}
