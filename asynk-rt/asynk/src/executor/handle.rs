use futures::{channel::oneshot, FutureExt};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug, thiserror::Error)]
#[error("join fail: result channel dropped")]
pub struct JoinError;

pub struct JoinHandle<T>(oneshot::Receiver<T>);

impl<T> JoinHandle<T>
where
    T: Send + 'static,
{
    pub(crate) fn new(rx: oneshot::Receiver<T>) -> Self {
        Self(rx)
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.poll_unpin(cx).map_err(|_| JoinError)
    }
}
