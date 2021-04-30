mod error;
mod http;
mod database;
mod application;


use std::{
    sync::{
        Arc
    }
};
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
    application::{
        Application
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

#[tokio::main]
async fn main() -> Result<(), FondyError> {
    // Подтягиваем окружение из файлика .env
    dotenv::dotenv().ok();

    // Инициализируем менеджер логирования
    initialize_logs();

    // База данных
    let db = Database::open_database()
        .await;

    // Шаблоны HTML
    let mut templates = handlebars::Handlebars::new();
    {
        templates.register_template_file("index", "templates/index.hbs")
            .expect("Index template read failed");
    }

    // Приложение со всеми нужными нам менеджерами
    let app = Arc::new(Application{
        db,
        templates
    });

    // Стартуем сервер
    start_server(app)
        .await;
    
    Ok(())
}