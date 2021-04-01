mod handlers;
mod models;

use actix_files::{
    Files
};
use actix_web::{
    App, 
    HttpServer, 
    guard::{
        self
    }, 
    web::{
        self
    }
};
use handlebars::{
    Handlebars
};
use tracing::{
    debug_span
};
use tracing_subscriber::{
    prelude::{
        *
    }
};
use tracing_actix_web::{
    TracingLogger
};
use crate::{
    handlers::{
        configure_routes
    }
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

struct LogGuards{
    _file_appender_guard: tracing_appender::non_blocking::WorkerGuard
}

fn initialize_logs() -> LogGuards{
    let (non_blocking_appender, _file_appender_guard) = 
        tracing_appender::non_blocking(tracing_appender::rolling::hourly("logs", "app_log"));
    let file_sub = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .json()
        .with_writer(non_blocking_appender);
    let stdoud_sub = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_thread_ids(true)
        .with_ansi(true)
        .with_writer(std::io::stdout);
    let full_subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::default()
                .add_directive(tracing::Level::TRACE.into())
                .and_then(file_sub))
        .with(tracing_subscriber::EnvFilter::from_default_env() // TODO: Почему-то все равно не работает
                .and_then(stdoud_sub));
    tracing::subscriber::set_global_default(full_subscriber).unwrap();

    LogGuards{
        _file_appender_guard
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Создаем менеджер шаблонов и регистрируем туда нужные
fn create_templates<'a>() -> Handlebars<'a> {
    let mut handlebars = Handlebars::new();    
    handlebars
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Инициализируем менеджер логирования
    let _log_guard = initialize_logs();

    // Базовый span для логирования
    let span = debug_span!("root_span");
    let _span_guard = span.enter();

    // Создаем шареную ссылку на обработчик шаблонов
    let handlebars = web::Data::new(create_templates());

    // Создаем общего http клиента для разных запросов
    let http_client = web::Data::new(reqwest::Client::new());

    HttpServer::new(move ||{
            // Приложение создается для каждого потока свое собственное
            // Порядок выполнения Middleware обратный, снизу вверх
            App::new()
                .wrap(TracingLogger)
                .app_data(handlebars.clone())
                .app_data(http_client.clone())
                .configure(configure_routes)
        }) 
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
