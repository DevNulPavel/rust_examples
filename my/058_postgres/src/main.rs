use tracing::{
    
};
use sqlx::{
    postgres::{
        PgPool,
        PgConnection
    }
};
use tracing_subscriber::{
    prelude::{
        *
    }
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn initialize_logs() {
    // Логи в stdout
    let stdoud_sub = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_thread_ids(true)
        .pretty()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_writer(std::io::stdout);

    // Суммарный обработчик
    let full_subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env() // TODO: Почему-то все равно не работает
                .and_then(stdoud_sub));

    // Установка по-умолчанию
    tracing::subscriber::set_global_default(full_subscriber).unwrap();
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() {
    // Вычитывем переменные окружения из файлика .env и добавляем их в окружение
    dotenv::dotenv()
        .ok();

    // Инициализируем менеджер логирования
    initialize_logs();

    // Подключаемся к базе
    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL does not exist");
    let pool = PgPool::connect(&db_url)
        .await
        .expect("Connection failed");

    // Выполняем миграции
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Database migrations failed");
}
