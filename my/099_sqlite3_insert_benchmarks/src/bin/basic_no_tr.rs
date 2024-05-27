use rusqlite::{params, Connection};
use sqlite3_insert_benchmarks as common;

fn faker(conn: Connection, count: i64) {
    for key in 0..count {
        // Генерируем рандомные значения
        let with_area = common::get_random_bool();
        let age = common::get_random_age();
        let is_active = common::get_random_active();

        if with_area {
            let area_code = common::get_random_area_code();
            conn.execute(
                "INSERT INTO user VALUES (?, ?, ?, ?)",
                params![key, area_code.as_str(), age, is_active],
            )
            .unwrap();
        } else {
            conn.execute(
                "INSERT INTO user VALUES (?, NULL, ?, ?)",
                params![key, age, is_active],
            )
            .unwrap();
        }
    }
}

fn main() {
    let conn = Connection::open("basic_no_tr.db").unwrap();
    conn.execute_batch(common::pragma_rules()).expect("PRAGMA");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user (
                id INTEGER not null primary key,
                area CHAR(6),
                age INTEGER not null,
                active INTEGER not null)",
        [],
    )
    .unwrap();
    faker(conn, 1_000_000)
}
