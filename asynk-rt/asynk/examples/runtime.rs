use futures::future;
use futures_timer::Delay;
use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

fn main() {
    asynk::builder().build().unwrap();
    asynk::block_on(main_future()).unwrap();
}

async fn main_future() {
    let val = Arc::new(AtomicU32::new(0));
    let expected_val = 10_000;

    let handles = (0..expected_val)
        .map(|_| Arc::clone(&val))
        .map(|val| {
            asynk::spawn(async move {
                // some computations ...
                Delay::new(Duration::from_secs(1)).await;
                val.fetch_add(1, Ordering::SeqCst);
            })
        })
        .collect::<Vec<_>>();

    future::join_all(handles).await;

    assert_eq!(val.load(Ordering::SeqCst), expected_val);
}
