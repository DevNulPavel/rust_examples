// use std::sync::Arc;
use rusqlite::{params, Connection};
use smallstr::SmallString;
use sqlite3_insert_benchmarks as common;

async fn faker(tx: tokio::sync::mpsc::Sender<Task>, total: usize) {
    let count = 16;
    let step = total / count;

    let joins: Vec<_> = (0..count)
        .map(|i| {
            let from = step * i;
            let to = (step * (i + 1)).min(total);
            let range = from..to;
            //dbg!(&range);
            let tx = tx.clone();
            tokio::spawn(async move {
                for key in range {
                    // Рандомные значения
                    let age = common::get_random_age();
                    let is_active = common::get_random_active();
                    let area_code = common::get_random_area_code();

                    let (resp, resp_r) = tokio::sync::oneshot::channel();
                    // let (resp, resp_r) = futures::channel::oneshot::channel();
                    // let notify = Arc::new(tokio::sync::Notify::new());

                    tx.send(Task {
                        key,
                        age,
                        is_active,
                        area_code,
                        resp,
                        // notify: notify.clone(),
                    })
                    .await
                    .unwrap();

                    // !!! WARNING !!!
                    // Больше всего времени тратится на ожидании результата здесь, так как async системе нужно проснуться
                    // let _res = resp_r.try_recv().ok();
                    let _new_tr = resp_r.await.unwrap();
                    // notify.notified().await;
                }
            })
        })
        .collect();

    for j in joins {
        j.await.unwrap();
    }
}

#[derive(Debug)]
struct Task {
    key: usize,
    age: i8,
    is_active: i8,
    area_code: SmallString<[u8; 6]>,
    resp: tokio::sync::oneshot::Sender<bool>,
    // resp: futures::channel::oneshot::Sender<bool>
    // notify: Arc<tokio::sync::Notify>,
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(64);

    let j = tokio::task::spawn_blocking(move || {
        let conn = Connection::open("basic_async_actor_mp.db").unwrap();
        println!("SQLite version: {}", rusqlite::version());
        conn.execute_batch(common::pragma_rules()).expect("PRAGMA");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user (
                    id INTEGER NOT NULL PRIMARY KEY,
                    area CHAR(6),
                    age INTEGER not null,
                    active INTEGER not null)",
            [],
        )
        .unwrap();

        let mut prepared_sql = conn
            .prepare_cached("INSERT INTO user VALUES (NULL, ?1, ?2, ?3)")
            .unwrap();

        // let mut prepared_select_sql = conn
        //     .prepare_cached("SELECT rowid, id, age FROM user WHERE id > ?1 LIMIT ?2")
        //     .unwrap();

        let mut tr = conn.unchecked_transaction().unwrap();

        // use std::time::{Duration, Instant};
        // let mut total_spent = Duration::default();

        while let Some(Task {
            key,
            age,
            is_active,
            area_code,
            resp,
            // notify
        }) = rx.blocking_recv()
        {
            prepared_sql
                .execute(params![area_code.as_str(), age, is_active])
                .unwrap();

            if key > 0 && key % 1_000 == 0 {
                tr.commit().unwrap();
                tr = conn.unchecked_transaction().unwrap();

                /*let start = Instant::now();
                let mut rows = prepared_select_sql.query(params![key, 1000]).unwrap();
                while let Some(row) = rows.next().unwrap() {
                    let _row_id: u64 = row.get(0).unwrap();
                    let _id: u64 = row.get(1).unwrap();
                    let _age: u32 = row.get(2).unwrap();
                    //println!("rowid: {_row_id}, id: {_id}, age: {_age}");
                }
                total_spent += start.elapsed();*/

                resp.send(true).ok();
                // notify.notify_one();
            } else {
                resp.send(false).ok();
                // notify.notify_one();
            }
        }

        tr.commit().unwrap();

        // println!("Time elapsed in SELECT SQL: {:?}", total_spent);
    });

    faker(tx, 1_000_000).await;

    j.await.unwrap();
}
