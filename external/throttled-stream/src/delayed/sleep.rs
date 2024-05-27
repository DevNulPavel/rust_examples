use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{ready, Context, Poll};
use std::time::Duration;
use tokio::time;
use tokio::time::{Instant, Sleep};

pin_project! {
    pub struct SleepDelay {
        dur: Duration,
        #[pin]
        sleep: Sleep
    }
}

impl SleepDelay {
    pub fn new(dur: Duration) -> Self {
        Self {
            dur,
            sleep: time::sleep(dur),
        }
    }
}

impl Future for SleepDelay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        ready!(this.sleep.as_mut().poll(cx));
        this.sleep.reset(Instant::now() + *this.dur);
        Poll::Ready(())
    }
}
