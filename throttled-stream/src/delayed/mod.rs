pub mod sleep;
pub mod tick;

use futures::{ready, Stream};
use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
    /// `Stream` с задержкой получения элементов
    pub struct DelayedStream<S, F> {
        // Внутренний стрим
        #[pin]
        inner_stream: S,
        #[pin]
        // Future, которая будет вызываться в рамках опроса стрима
        delay_fut: F,
    }
}

impl<S, F> DelayedStream<S, F> {
    pub fn new(inner_stream: S, delay_fut: F) -> Self {
        Self {
            inner_stream,
            delay_fut,
        }
    }
}

impl<S: Stream, F: Future> Stream for DelayedStream<S, F> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        // Проверяем, что ожидание завершено. Если нет - возвращаем `Poll::Pending`
        ready!(this.delay_fut.as_mut().poll(cx));
        this.inner_stream.poll_next(cx)
    }
}
