use sqlite3_insert_benchmarks as common;
use sqlx::sqlite::{
    SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteLockingMode, SqliteSynchronous,
};
use sqlx::{ConnectOptions, Connection, Executor, SqliteConnection, Statement};
use std::time::Duration;

async fn faker(mut conn: SqliteConnection, count: i64) -> Result<(), sqlx::Error> {
    let mut tx = conn.begin().await?;
    let stmt_with_area = tx
        .prepare("INSERT INTO user VALUES (NULL, ?, ?, ?)")
        .await?;
    let stmt = tx
        .prepare("INSERT INTO user VALUES (NULL, NULL, ?, ?)")
        .await?;
    for _ in 0..count {
        let with_area = common::get_random_bool();
        let age = common::get_random_age();
        let is_active = common::get_random_active();
        if with_area {
            let area_code = common::get_random_area_code();
            stmt_with_area
                .query()
                .bind(area_code.as_str())
                .bind(age)
                .bind(is_active)
                .execute(&mut tx)
                .await?;
        } else {
            stmt.query()
                .bind(age)
                .bind(is_active)
                .execute(&mut tx)
                .await?;
        }
    }
    tx.commit().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let mut conn = SqliteConnectOptions::new()
        .filename(":memory:")
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .auto_vacuum(SqliteAutoVacuum::None)
        .foreign_keys(true)
        .locking_mode(SqliteLockingMode::Normal)
        .shared_cache(true)
        .busy_timeout(Duration::from_secs(600))
        .pragma("temp_store", "memory")
        .pragma("cache_size", "-524288")
        .pragma("wal_autocheckpoint", "0")
        .disable_statement_logging()
        .connect()
        .await?;

    conn.execute(common::pragma_rules()).await?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user (
                id INTEGER not null primary key,
                area CHAR(6),
                age INTEGER not null,
                active INTEGER not null);",
    )
    .await?;
    faker(conn, 1_000_000).await?;
    Ok(())
}
