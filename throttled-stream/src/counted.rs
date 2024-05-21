use futures::Stream;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{ready, Context, Poll};

pin_project! {
    pub struct CountedStream<S> {
        #[pin]
        inner_stream: S,
        count: usize,
        polled: AtomicUsize,
    }
}

impl<S> CountedStream<S> {
    pub fn new(inner_stream: S, count: usize) -> Self {
        Self {
            inner_stream,
            count,
            polled: AtomicUsize::new(0),
        }
    }
}

impl<S: Stream> Stream for CountedStream<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        if this.polled.load(Ordering::SeqCst) >= *this.count {
            return Poll::Ready(None);
        }

        let item = ready!(this.inner_stream.poll_next(cx));
        this.polled.fetch_add(1, Ordering::SeqCst);
        Poll::Ready(item)
    }
}
