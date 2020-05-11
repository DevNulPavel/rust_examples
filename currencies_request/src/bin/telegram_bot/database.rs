use std::{
    path::{
        Path,
        // PathBuf
    }
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
    const FILE_NAME: &str = "telegram_bot.sqlite";

    if tokio::fs::File::open(FILE_NAME).await.is_err(){
        tokio::fs::File::create(FILE_NAME).await.expect("Database file create failed");
    }

    // База данных
    let mut db_conn = SqliteConnection::connect("sqlite:./telegram_bot.sqlite")
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