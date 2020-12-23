mod database;
mod custom_db;
mod mm_db;

use tokio::{
    runtime::{
        Builder
    }
};

async fn async_main(){
    mm_db::test_mm_db().await;
}

fn main() {
    let mut runtime = Builder::new()
        .threaded_scheduler()
        .enable_io()
        .build()
        .expect("Runtime build failed");

    runtime.block_on(async_main());
}