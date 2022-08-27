use rusqlite::{params, Connection};
use sqlite3_insert_benchmarks as common;

async fn faker(tx: tokio::sync::mpsc::Sender<Task>, count: usize) {
    for key in 0..count {
        // Рандомные значения
        let age = common::get_random_age();
        let is_active = common::get_random_active();
        let area_code = common::get_random_area_code();

        let (resp, resp_r) = tokio::sync::oneshot::channel();

        tx.send(Task {
            key,
            age,
            is_active,
            area_code,
            resp,
        })
        .await
        .unwrap();

        // Больше всего времени тратится на ожидании результата здесь, так как async системе нужно проснуться
        // let _res = resp_r.try_recv().ok();
        let _new_tr = resp_r.await.unwrap();
    }
}

#[derive(Debug)]
struct Task {
    key: usize,
    age: i8,
    is_active: i8,
    area_code: String,
    resp: tokio::sync::oneshot::Sender<bool>,
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);

    let j = tokio::task::spawn_blocking(move || {
        let conn = Connection::open("basic_async_actor.db").unwrap();
        conn.execute_batch(common::pragma_rules()).expect("PRAGMA");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user (
                    id INTEGER not null primary key AUTOINCREMENT,
                    area CHAR(6),
                    age INTEGER not null,
                    active INTEGER not null)",
            [],
        )
        .unwrap();

        let mut prepared_sql = conn
            .prepare_cached("INSERT INTO user VALUES (?, ?, ?, ?)")
            .unwrap();

        let mut tr = conn.unchecked_transaction().unwrap();

        while let Some(Task {
            key,
            age,
            is_active,
            area_code,
            resp,
        }) = rx.blocking_recv()
        {
            prepared_sql
                .execute(params![key, area_code, age, is_active])
                .unwrap();

            if key > 0 && key % 500 == 0 {
                tr.commit().unwrap();
                tr = conn.unchecked_transaction().unwrap();

                resp.send(true).ok();
            } else {
                resp.send(false).ok();
            }
        }

        tr.commit().unwrap();
    });

    faker(tx, 1_000_000).await;

    j.await.unwrap();
}
