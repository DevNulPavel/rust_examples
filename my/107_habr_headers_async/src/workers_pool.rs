/*use rayon::{ThreadPool, ThreadPoolBuilder};
use std::{
    sync::{
        Arc,
        mpsc
    }
};
use tokio::{sync::oneshot, task::block_in_place};

pub struct WorkersPool {
    thread_pool: Arc<ThreadPool>,
}

impl Clone for WorkersPool {
    fn clone(&self) -> Self {
        WorkersPool {
            thread_pool: self.thread_pool.clone(),
        }
    }
}

impl WorkersPool {
    #[cfg_attr(feature = "flame_it", flamer::flame)]
    pub fn new(num_threads: usize) -> WorkersPool {
        let thread_pool = Arc::new(
            ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
                .expect("Thread pool create failed"),
        );

        WorkersPool { thread_pool }
    }

    #[cfg_attr(feature = "flame_it", flamer::flame)]
    pub async fn queue_task<T, R>(&self, task: T) -> R
    where
        T: 'static + Send + FnOnce() -> R,
        R: 'static + Send,
    {
        // let (task_sender, task_receiver) = mpsc::channel::<T>();
        let (res_sender, res_receiver) = oneshot::channel();

        self.thread_pool.spawn(move || {
            let result = task();

            match res_sender.send(result) {
                Ok(_) => {}
                Err(_) => {
                    panic!("Result send failed");
                }
            }
        });

        res_receiver.await.expect("Thread pool receive result failed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_1() {
        let pool = WorkersPool::new(4);
        let future = pool.queue_task(|| {
            println!("Success");
            1
        });

        let result = future.await;

        assert_eq!(result, 1);
    }
}*/