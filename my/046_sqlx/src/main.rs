//use std::env;
use sqlx::{
    prelude::*,
    postgres::{
        //PgQueryAs,
        PgPool,
        PgRow,
        PgCursor
    },
    sqlite::{
        Sqlite,
        SqliteConnection,
        SqliteCursor,
        SqliteRow
        //SqlitePool,
        //SqliteQueryAs
    }
};
use tokio::stream::{
    Stream,
    StreamExt,
};
// use sqlx::mysql::MySqlPool;

async fn postgres_simple_example() -> Result<(), sqlx::Error>{
    //let database_url = env::var("DATABASE_URL")?;
    let database_url = "127.0.0.1";
    
    // Создаем пул соединений
    let pool = PgPool::builder()
        .max_size(5) // Максимальное количество соединений в пуле
        .build(database_url)
        .await?;
    
    // Выполняем простой запрос имеющий параметры
    let mut cursor: PgCursor = sqlx::query("SELECT $1")
        .bind(150_i64)
        .fetch(&pool);

    while let Ok(row) = cursor.next().await {
        let row: Option<PgRow> = row;
        if let Some(row) = row{
            let val: i64 = row.get(0);
            println!("Val = {}", val);
            assert_eq!(val, 150);
        }
    }

    // let query = sqlx::query_as("SELECT $1")
    //     .bind(150_i64);

    // sqlx::query("DELETE FROM table")
    //     .execute(&pool)
    //     .await?;
    
    Ok(())
}

async fn sqlite_simple_example() -> Result<(), sqlx::Error> {
    let mut conn = SqliteConnection::connect("sqlite::memory:")
        .await?;

    // Выполняем простой запрос имеющий параметры
    {
        let mut cursor: SqliteCursor = sqlx::query("SELECT $1")
            .bind(150_i64)
            .fetch(&mut conn);

        'while_loop: while let Ok(row) = cursor.next().await {
            let row: Option<SqliteRow> = row;
            if let Some(row) = row{
                let val: i64 = row.get(0);
                println!("Val = {}", val);
                assert_eq!(val, 150);
                
                break 'while_loop;
            }
        }        
    }

    {
        #[derive(sqlx::FromRow)]
        struct Val { 
            val: i64 
        }

        // Можно запрос превращать сразу в поток наших данных
        let mut stream = sqlx::query_as::<Sqlite, Val>("SELECT $1")
            .bind(123_i64)
            .fetch(&mut conn);

        for val in stream.next().await {

        }

        // Можно запрос превращать сразу в поток наших данных
        /*let stream = sqlx::query("SELECT $1")
            .bind(123_i64)
            .map(|row: SqliteRow| {
                // Превращаем
                let val: i64 = row.get(0);
                val
            })
            .fetch(&mut conn);*/
        
        /*stream
            .take(1)
            .for_each(|item| {
            })
            .await;*/
    }

    Ok(())
}

#[tokio::main] // #[async_std::main] // or #[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    postgres_simple_example()
        .await?;
    
    sqlite_simple_example()
        .await?;

    Ok(())
}