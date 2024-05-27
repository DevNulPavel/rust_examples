use postgres::{
    config::{Config, SslMode, TargetSessionAttrs},
    types::ToSql,
    Client, IsolationLevel, NoTls,
};
use smallstr::SmallString;
use sqlite3_insert_benchmarks as common;
use std::time::Duration;

fn faker(mut conn: Client, count: i64) {
    // Запускаем транзакцию
    let mut tr = conn
        .build_transaction()
        .isolation_level(IsolationLevel::ReadCommitted)
        .start()
        .unwrap();

    let q = tr
        .prepare("INSERT INTO users(area, age, active) VALUES ($1, $2, $3);")
        .unwrap();

    for _ in 0..count {
        // Генерируем рандомные значения
        let age: i16 = common::get_random_age() as i16;
        let is_active: bool = common::get_random_active() == 0;
        let area_code: SmallString<[u8; 6]> = common::get_random_area_code();

        // Выполняем с параметрами
        let params: &[&(dyn ToSql + Sync)] = &[&area_code.as_str(), &age, &is_active];
        tr.execute(&q, params).unwrap();
    }

    // Завершаем транзакцию
    tr.commit().unwrap();
}

fn main() {
    let mut client = Config::new()
        .application_name("test_app")
        .dbname("bench_test")
        .host("127.0.0.1")
        .port(5432)
        .keepalives(true)
        .user("devnul")
        .password("")
        .target_session_attrs(TargetSessionAttrs::Any)
        .keepalives_idle(Duration::from_secs(30))
        .ssl_mode(SslMode::Disable)
        .connect(NoTls)
        .unwrap();

    // Создаем табличку, удалив старую
    client.execute("DROP TABLE IF EXISTS users", &[]).unwrap();
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                area CHAR(6),
                age SMALLINT NOT NULL,
                active BOOL NOT NULL
            )",
            &[],
        )
        .unwrap();

    faker(client, 1_000_000);
}
