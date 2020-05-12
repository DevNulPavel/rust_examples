use std::{
    path::{
        Path,
        // PathBuf
    },
    env
};
use sqlx::{
    // prelude::*,
    // query,
    Connect,
    // Connection,
    // Executor,
    Cursor,
    sqlite::{
        SqliteConnection,
        SqliteCursor,
        SqliteRow
    }
};
use tokio::{
    fs::{
        read_to_string
    }
};



async fn build_tables(conn: &mut SqliteConnection){
    let db_path = Path::new("sql/create_database.sql");
    let sql_text: String = read_to_string(db_path)
        .await
        .expect("File сreate_database.sql does not exist");

    sqlx::query(&sql_text)
        .execute(conn)
        .await
        .expect("Create database failed");
}

async fn check_tables_exists(conn: &mut SqliteConnection) -> bool {
    const SQL: &str = "SELECT name \
                       FROM sqlite_master \
                       WHERE type='table' AND name='monitoring_users'";
    let mut cursor: SqliteCursor = sqlx::query(SQL)
        .fetch(conn);
    
    let value: Option<SqliteRow> = cursor.next().await.expect("Cursor error");

    value.is_some()
}

pub async fn get_database() -> SqliteConnection {
    // TODO: Improve
    let database_path: std::path::PathBuf = if cfg!(debug_assertions) {
        const FILE_NAME: &str = "database/telegram_bot.sqlite";
        let database_path: String = env::var("TELEGRAM_DATABASE_PATH").unwrap_or(FILE_NAME.into());
        println!("TELEGRAM_DATABASE_PATH: {}", database_path);
        database_path.into()
    }else{
        let database_path: String = env::var("TELEGRAM_DATABASE_PATH").expect("TELEGRAM_DATABASE_PATH not set");
        println!("TELEGRAM_DATABASE_PATH: {}", database_path);
        database_path.into()
    };

    if database_path.exists() == false{
        let folder = database_path.parent().expect("Database path get failed");
        tokio::fs::create_dir(folder).await.ok();
        tokio::fs::File::create(&database_path).await.expect("Database file create failed");
    }

    // База данных
    let connect_path: String = format!("sqlite:{}", database_path.to_str().expect("Db path to string failed"));
    let mut db_conn = SqliteConnection::connect(connect_path)
        .await
        .expect("Sqlite connection create failed");

    let tables_exist = check_tables_exists(&mut db_conn).await;
    if !tables_exist {
        println!("Database doesn't exist, need to create it");
        build_tables(&mut db_conn).await;
    }else{
        println!("Database already exists");
    }

    db_conn
}