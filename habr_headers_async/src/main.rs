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
    let mut runtime = Builder::default()
        .enable_io() 
        .basic_scheduler()
        .build()
        .expect("Tokio runctime create failed");

    runtime.block_on(async_main());
}