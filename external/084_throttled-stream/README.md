# throttled-stream
Данная библиотека представляет расширения для `Stream` с задержкой возврата значения, а также ограничения количества элементов
```rust
pub trait ThrottledStreamExt<S: Stream> {
    /// Получить `Stream`, который отдаст максимально возможное количество элементов
    fn max(self, count: usize) -> CountedStream<S>;

    /// Ожидание (засыпание) промежутка времени `dur` между отда элементов `Stream`'а
    fn sleep(self, dur: Duration) -> DelayedStream<S, SleepDelay>;

    /// Интервальное ожидание между отдачами элемента. Это значит, что элементы будут выдаваться
    /// **не чаще**, чем указанный `Duration`.
    fn tick(self, dur: Duration) -> DelayedStream<S, TickDelay>;
}
```

## Example
```rust
use throttled_stream::ThrottledStreamExt;
use futures::{stream, StreamExt};
use std::pin::pin;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let stream = stream::iter(vec![1, 3, 2, 4, 5])
        .tick(Duration::from_secs(1))
        .max(4);

    let mut stream = pin!(stream);

    let mut count = 0;

    while let Some(v) = stream.next().await {
        // elapsed 1 sec ...
        println!("{}", v);
        count += 1;
    }

    assert_eq!(count, 4);
}
```