mod error;
mod client;
mod page;
mod article;
mod workers_pool;

use tokio::{
    runtime::{
        Builder
    }
};


async fn async_main() {
}

fn main(){
    let runtime = Builder::new_current_thread()
        .enable_io()
        .max_threads(8) // TODO: Num cpus
        .worker_threads(1) 
        .build()
        .expect("Tokio runtime create failed");

    runtime.block_on(async_main());
}