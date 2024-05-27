use crate::tp::ThreadPool;
use std::{sync::mpsc, thread, time::Duration};

#[test]
fn test_thread_pool() {
    let thread_count = 4;

    let tp = ThreadPool::new("test".into(), thread_count);

    let (tx, rx) = mpsc::channel();

    for _ in 0..thread_count {
        let tx = tx.clone();
        tp.spawn(move || {
            tx.send(1).unwrap();
        });
    }

    assert_eq!(rx.iter().take(thread_count).sum::<usize>(), thread_count);
}

#[test]
fn test_thread_pool_panic() {
    let tp = ThreadPool::new("test".into(), 4);

    tp.spawn(|| {
        thread::sleep(Duration::from_secs(1));
        panic!("boom");
    });

    thread::sleep(Duration::from_secs(5));
}
