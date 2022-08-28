use rusqlite::{params, Connection};

use sqlite3_insert_benchmarks as common;

fn faker_wrapper(conn: Connection, count: i64) {
    // Работа
    // Заранее подготавливаем запросы
    let mut stmt_with_area = conn
        .prepare_cached("INSERT INTO user VALUES (?, ?, ?, ?)")
        .unwrap();
    let mut stmt = conn
        .prepare_cached("INSERT INTO user VALUES (?, NULL, ?, ?)")
        .unwrap();
    // let mut update = tx
    //     .prepare_cached("UPDATE user SET age = ? WHERE id = ?")
    //     .unwrap();

    // Запуск транзакции
    let mut tr = conn.unchecked_transaction().unwrap();

    for key in 0..count {
        // Рандомные значения
        let with_area = common::get_random_bool();
        let age = common::get_random_age();
        let is_active = common::get_random_active();

        if with_area {
            let area_code = common::get_random_area_code();
            stmt_with_area
                .execute(params![key, area_code, age, is_active])
                .unwrap();
        } else {
            stmt.execute(params![key, age, is_active]).unwrap();
        }

        // let age = common::get_random_age();
        // update.execute(params![age, key]).unwrap();

        if key > 0 && key % 1_000 == 0 {
            tr.commit().unwrap();
            tr = conn.unchecked_transaction().unwrap();
            // let row_id = tr.last_insert_rowid();
            // println!("{row_id}");
        }
    }

    // Завершение транзакции
    tr.commit().unwrap();
}

fn main() {
    let conn = Connection::open("basic_prep_raw_tr.db").unwrap();
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
    faker_wrapper(conn, 1_000_000)
}
