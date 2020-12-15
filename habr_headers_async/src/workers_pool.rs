/*use std::{
    fmt::{
       Debug 
    }
};
use rayon::{
    ThreadPool,
    ThreadPoolBuilder
};
use tokio::{
    sync::{
        oneshot
    }  
};

pub struct WorkersPool{
    thread_pool: ThreadPool
}

impl<'a> WorkersPool {
    pub fn new(num_threads: usize) -> WorkersPool{
        let thread_pool = ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .expect("Thread pool create failed");

        WorkersPool{
            thread_pool
        }
    } 
    pub async fn add_task<T, R>(&self, task: T) -> R
    where 
        T: Sized + Send + FnOnce()->R + 'a,
        R: Sized + Send + Debug {

        let (sender, receiver) = oneshot::channel();
        let internal_task = move || {
            let result = task();
            sender
                .send(result)
                .expect("Result send failed");
        };
        
        self.thread_pool
            .spawn();
        receiver
            .await
            .expect("Thread pool receive result failed")
    }
}*/