mod counted;
mod delayed;

use crate::counted::CountedStream;
use crate::delayed::sleep::SleepDelay;
use crate::delayed::tick::TickDelay;
use crate::delayed::DelayedStream;
use futures::Stream;
use std::time::Duration;

/// Расширение для `Stream`, которое позволяет ограничивать/замедлять `Stream`
/// ```
/// use throttled_stream::ThrottledStreamExt;
/// use futures::{stream, StreamExt};
/// use std::pin::pin;
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() {
///     let stream = stream::iter(vec![1, 3, 2, 4, 5])
///         .tick(Duration::from_secs(1))
///         .max(4);
///
///     let mut stream = pin!(stream);
///
///     let mut count = 0;
///
///     while let Some(v) = stream.next().await {
///         // elapsed 1 sec ...
///         println!("{}", v);
///         count += 1;
///     }
///
///     assert_eq!(count, 4);
/// }
/// ```
pub trait ThrottledStreamExt<S: Stream> {
    /// Получить `Stream`, который отдаст максимально возможное количество элементов
    fn max(self, count: usize) -> CountedStream<S>;

    /// Ожидание (засыпание) промежутка времени `dur` между отда элементов `Stream`'а
    fn sleep(self, dur: Duration) -> DelayedStream<S, SleepDelay>;

    /// Интервальное ожидание между отдачами элемента. Это значит, что элементы будут выдаваться
    /// **не чаще**, чем указанный `Duration`.
    fn tick(self, dur: Duration) -> DelayedStream<S, TickDelay>;
}

impl<S: Stream> ThrottledStreamExt<S> for S {
    fn max(self, count: usize) -> CountedStream<S> {
        CountedStream::new(self, count)
    }

    fn sleep(self, dur: Duration) -> DelayedStream<S, SleepDelay> {
        DelayedStream::new(self, SleepDelay::new(dur))
    }

    fn tick(self, dur: Duration) -> DelayedStream<S, TickDelay> {
        DelayedStream::new(self, TickDelay::new(dur))
    }
}

#[cfg(test)]
mod tests {
    use crate::ThrottledStreamExt;
    use futures::{stream, StreamExt};

    #[tokio::test]
    async fn max() {
        let stream = stream::iter(vec![1, 3, 2, 4, 5]).max(3);
        assert_eq!(stream.collect::<Vec<_>>().await.len(), 3);
    }
}
