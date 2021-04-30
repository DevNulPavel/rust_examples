use sqlx::{
    sqlite::{
        SqlitePoolOptions,
        SqlitePool
    },
    // migrate
};
use tracing::{
    instrument,
    debug
};
use crate::{
    error::{
        FondyError
    }
};

#[derive(Debug)]
pub struct Database{
    pool: SqlitePool
}

impl Database {
    /// Открывает базу данных и выполняет миграцию
    #[instrument]
    pub async fn open_database() -> Database {
        let db_url = std::env::var("DATABASE_URL")
            .expect("Missing DATABASE_URL variable");

        // Создаем файлик с пустой базой данных если его нету
        {
            const PREFIX: &str = "sqlite://";
            assert!(db_url.starts_with(PREFIX), "DATABASE_URL must stats with {}", PREFIX);

            let file_path = std::path::Path::new(db_url.trim_start_matches(PREFIX));
            if !file_path.exists() {
                if let Some(dir) = file_path.parent(){
                    std::fs::create_dir_all(dir)
                        .expect("Database directory create failed");
                }
                std::fs::File::create(file_path)
                    .expect("Database file create failed");
            }
        }

        // Пулл соединений
        let pool = SqlitePoolOptions::new()
            .connect(&db_url)
            .await
            .expect("Database connection failed");
        debug!("Database pool created");
        
        // Миграция базы
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("SQLx migration failed");
        debug!("Migration complete");

        Database{
            pool
        }
    }
}