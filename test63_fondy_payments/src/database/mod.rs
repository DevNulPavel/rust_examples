use sqlx::{
    sqlite::{
        SqlitePoolOptions,
        SqlitePool
    },
    // migrate
};
use crate::{
    error::{
        FondyError
    }
};

pub struct Database{
    pool: SqlitePool
}

impl Database {
    pub async fn open_database() -> Database {
        let db_url = std::env::var("DATABASE_URL")
            .expect("Missing DATABASE_URL variable");

        let pool = SqlitePoolOptions::new()
            .connect(&db_url)
            .await
            .expect("Database connection failed");
        
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("SQLx migration failed");

        Database{
            pool
        }
    }
}