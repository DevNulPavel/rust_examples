use async_trait::async_trait;
use deadpool::managed::{Manager, Pool, RecycleResult};
use flume::{bounded, Receiver};
use rusqlite::{params, Connection};
use sqlite3_insert_benchmarks as common;
use std::{path::PathBuf, sync::Mutex};

struct SQLiteManager {
    path: PathBuf,
}

#[async_trait]
impl Manager for SQLiteManager {
    type Type = Mutex<Connection>;
    type Error = rusqlite::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let conn = Connection::open(&self.path)?;
        conn.set_prepared_statement_cache_capacity(128);
        conn.execute_batch(common::pragma_rules())?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user (
                    id INTEGER not null primary key AUTOINCREMENT,
                    area CHAR(6),
                    age INTEGER not null,
                    active INTEGER not null)",
            [],
        )?;
        Ok(Mutex::new(conn))
    }

    async fn recycle(&self, _: &mut Self::Type) -> RecycleResult<Self::Error> {
        Ok(())
    }

    fn detach(&self, _obj: &mut Self::Type) {}
}

async fn executor(pool: Pool<SQLiteManager>, rx: Receiver<usize>) {
    // Запускаем транзакцию
    let conn = pool.get().await.unwrap();

    while let Ok(_i) = rx.recv_async().await {
        let mut conn_lock = conn.lock().unwrap();
        let tr = conn_lock.transaction().unwrap();

        // Генерируем рандомные значения
        let with_area = common::get_random_bool();
        let age = common::get_random_age();
        let is_active = common::get_random_active();

        if with_area {
            let area_code = common::get_random_area_code();
            tr.prepare("INSERT INTO user VALUES (NULL, ?, ?, ?)")
                .unwrap()
                .execute(params![area_code.as_str(), age, is_active])
                .unwrap();
        } else {
            tr.prepare("INSERT INTO user VALUES (NULL, NULL, ?, ?)")
                .unwrap()
                .execute(params![age, is_active])
                .unwrap();
        }

        // Завершаем транзакцию
        tr.commit().unwrap();
    }
}

#[tokio::main]
async fn main() {
    let manager = SQLiteManager {
        path: PathBuf::from("basic_deadpool.db"),
    };
    let pool = Pool::builder(manager)
        .create_timeout(None)
        .max_size(16)
        .build()
        .unwrap();

    let (tx, rx) = bounded(64);

    let joins: Vec<_> = (0..16)
        .map(|_| {
            let pool = pool.clone();
            let rx = rx.clone();
            tokio::spawn(executor(pool, rx))
        })
        .collect();

    for i in 0..1_000_000 {
        tx.send(i).unwrap();
    }

    for join in joins {
        join.await.unwrap();
    }
}
