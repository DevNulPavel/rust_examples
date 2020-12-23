mod database;

use std::{
    path::{
        Path
    }
};
use tokio::{
    runtime::{
        Builder
    }
};
use sqlx::{
    sqlite::{
        SqliteConnection,
        // SqlitePool
    },
    // Connection,
};
use database::{
    Database
};

async fn async_main(){
    let db = Database::open(Path::new("database.sqlite"))
        .await
        .expect("Database open failed");

    let db: SqliteConnection = db.into();
}

fn main() {
    let mut runtime = Builder::new()
        .threaded_scheduler()
        .enable_io()
        .build()
        .expect("Runtime build failed");

    runtime.block_on(async_main());
}