use mysql::{prelude::Queryable, Pool, PooledConn, TxOpts};
use smallstr::SmallString;
use sqlite3_insert_benchmarks as common;

fn faker(mut conn: PooledConn, count: i64) {
    // Запускаем транзакцию
    let mut tr = conn.start_transaction(TxOpts::default()).unwrap();

    let q = tr
        .prep("INSERT INTO users(area, age, active) VALUES (?, ?, ?);")
        .unwrap();

    for _ in 0..count {
        // Генерируем рандомные значения
        let age: i16 = common::get_random_age() as i16;
        let is_active: bool = common::get_random_active() == 0;
        let area_code: SmallString<[u8; 6]> = common::get_random_area_code();

        // Выполняем с параметрами
        tr.exec_drop(q.clone(), (area_code.as_str(), age, is_active))
            .unwrap();
    }

    // Завершаем транзакцию
    tr.commit().unwrap();
}

fn main() {
    let opts = mysql::OptsBuilder::new()
        .compress(None)
        .user(Some("root"))
        .db_name(Some("test_db"))
        .ip_or_hostname(Some("127.0.0.1"))
        .tcp_port(3306);
    let client = Pool::new(opts).unwrap();

    let mut conn = client.get_conn().unwrap();

    // Создаем табличку, удалив старую
    conn.exec_drop("DROP TABLE IF EXISTS users", ()).unwrap();
    conn.exec_drop(
        "CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                area CHAR(6),
                age SMALLINT NOT NULL,
                active BOOL NOT NULL
            )",
        (),
    )
    .unwrap();

    faker(conn, 1_000_000);
}
