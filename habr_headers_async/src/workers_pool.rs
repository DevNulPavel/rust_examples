use std::{
    sync::{
        Arc
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
    thread_pool: Arc<ThreadPool> 
}

impl Clone for WorkersPool {
    fn clone(&self) -> Self {
        WorkersPool{
            thread_pool: self.thread_pool.clone()
        }
    }
}

impl WorkersPool {
    pub fn new(num_threads: usize) -> WorkersPool{
        let thread_pool = Arc::new(ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .expect("Thread pool create failed"));

        WorkersPool{
            thread_pool
        }
    } 

    pub async fn queue_task<T, R>(&self, task: T) -> R
    where 
        // 'static для замыкания значит, что замыкание может иметь лишь ссылки на 'static, 
        // остальное должно быть move в замыкание
        T: 'static + Send + FnOnce()->R,
        R: 'static + Send {

        let (sender, receiver) = oneshot::channel();
        
        self.thread_pool
            .spawn(move || {
                let result = task();
                match sender.send(result){
                    Ok(_) => {},
                    Err(_) => {
                        panic!("Result send failed");
                    }
                }
            });
        receiver
            .await
            .expect("Thread pool receive result failed")
    }
}


#[cfg(test)]
mod tests{
    use super::*;

    #[tokio::test]
    async fn test_pool_1(){
        let pool = WorkersPool::new(4);
        let future = pool.queue_task(||{
            println!("Success");
            1
        });

        let result = future.await;
        
        assert_eq!(result, 1);
    }
}