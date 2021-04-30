mod error;
mod http;
mod database;

use tracing::{
    debug_span,
    debug,
    event,
    instrument,
    Level,
};
use tracing_subscriber::{
    prelude::{
        *
    },
    fmt::{
        format::{
            FmtSpan
        }
    }
};
use warp::{
    Filter,
    Reply,
    Rejection
};
use crate::{
    http::{
        start_server
    },
    database::{
        Database
    },
    error::{
        FondyError
    }
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn initialize_logs() {
    // Логи в stdout
    let stdoud_sub = tracing_subscriber::fmt::layer()
        .pretty()
        .with_writer(std::io::stdout)
        .with_span_events(FmtSpan::FULL);

    // Суммарный обработчик
    let full_subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env()
                .and_then(stdoud_sub));

    // Установка по-умолчанию
    tracing::subscriber::set_global_default(full_subscriber)
        .unwrap();
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////


#[tokio::main]
async fn main() -> Result<(), FondyError> {
    // Подтягиваем окружение из файлика .env
    dotenv::dotenv().ok();

    // Инициализируем менеджер логирования
    initialize_logs();

    // База данных
    let db = Database::open_database()
        .await;

    // Стартуем сервер
    start_server()
        .await;
    
    Ok(())
}