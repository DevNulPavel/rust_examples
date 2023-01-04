use smallstr::SmallString;
use sqlite3_insert_benchmarks as common;
use std::time::Duration;
use tokio_postgres::{
    config::{Config, SslMode, TargetSessionAttrs},
    types::ToSql,
    Client, IsolationLevel, NoTls,
};

async fn faker(mut conn: Client, count: i64) {
    // Запускаем транзакцию
    let tr = conn
        .build_transaction()
        .isolation_level(IsolationLevel::ReadCommitted)
        .start()
        .await
        .unwrap();

    let q = tr
        .prepare("INSERT INTO users(area, age, active) VALUES ($1, $2, $3);")
        .await
        .unwrap();

    for _ in 0..count {
        // Генерируем рандомные значения
        let age: i16 = common::get_random_age() as i16;
        let is_active: bool = common::get_random_active() == 0;
        let area_code: SmallString<[u8; 6]> = common::get_random_area_code();

        // Выполняем с параметрами
        let params: &[&(dyn ToSql + Sync)] = &[&area_code.as_str(), &age, &is_active];
        tr.execute(&q, params).await.unwrap();
    }

    // Завершаем транзакцию
    tr.commit().await.unwrap();
}

#[tokio::main]
async fn main() {
    let (client, connection) = Config::new()
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
        .await
        .unwrap();

    // Объект соединения выполняет коммуникацию с базой данных.
    // Запускать нужно в отдельной корутине.
    // Может быть можно делать переподключение там же внутри?
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Создаем табличку, удалив старую
    client
        .execute("DROP TABLE IF EXISTS users", &[])
        .await
        .unwrap();
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
        .await
        .unwrap();

    faker(client, 1_000_000).await;
}
