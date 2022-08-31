use rusqlite::{params, Connection};
use sqlite3_insert_benchmarks as common;

fn faker(conn: Connection, count: i64) {
    let mut tr = conn.unchecked_transaction().unwrap();
    tr.set_drop_behavior(rusqlite::DropBehavior::Commit);
    // conn.execute("BEGIN TRANSACTION", []).unwrap();
    for key in 0..count {
        // Генерируем рандомные значения
        let with_area = common::get_random_bool();
        let age = common::get_random_age();
        let is_active = common::get_random_active();

        if with_area {
            let area_code = common::get_random_area_code();
            tr.execute(
                "INSERT INTO user VALUES (?, ?, ?, ?)",
                params![key, area_code, age, is_active],
            )
            .unwrap();
        } else {
            tr.execute(
                "INSERT INTO user VALUES (?, NULL, ?, ?)",
                params![key, age, is_active],
            )
            .unwrap();
        }

        let _val: i32 = tr
            .query_row("SELECT age FROM user WHERE id = ?", params![key], |r| {
                r.get(0)
            })
            .unwrap();

        if key > 0 && key % 10_000 == 0 {
            tr.commit().unwrap();
            tr = conn.unchecked_transaction().unwrap();
            // conn.execute("COMMIT TRANSACTION", []).unwrap();
            // conn.execute("BEGIN TRANSACTION", []).unwrap();
        }
    }
    tr.commit().unwrap();
    // conn.execute("COMMIT TRANSACTION", []).unwrap();
}

fn main() {
    let conn = Connection::open("basic.db").unwrap();
    conn.execute_batch(common::pragma_rules()).expect("PRAGMA");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user (
                id INTEGER not null PRIMARY KEY,
                area CHAR(6),
                age INTEGER not null,
                active INTEGER not null)",
        [],
    )
    .unwrap();
    faker(conn, 1_000_000);
}
