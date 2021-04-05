mod handlers;
mod models;
mod error;
mod crypto;

use actix_web::{
    App, 
    HttpServer, 
    web::{
        self
    }
};
use tracing::{
    debug_span,
    debug,
    event,
    instrument,
    Level
};
use tracing_subscriber::{
    prelude::{
        *
    }
};
use tracing_actix_web::{
    TracingLogger
};
use sqlx::{
    PgPool
};
use crate::{
    handlers::{
        configure_routes
    },
    crypto::{
        TokenService
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

#[instrument(name = "database_open")]
pub async fn open_database() -> PgPool {
    let pg_conn = PgPool::connect(&std::env::var("DATABASE_URL")
                                    .expect("DATABASE_URL env variable is missing"))
        .await
        .expect("Database connection failed");

    event!(Level::DEBUG, 
            db_type = %"pg", // Будет отформатировано как Display
            "Database open success");

    // Включаем все миграции базы данных сразу в наш бинарник, выполняем при старте
    sqlx::migrate!("./migrations")
        .run(&pg_conn)
        .await
        .expect("Database migration failed");

    debug!(migration_file = ?"./migrations", // Будет отформатировано как debug
            "Database migration finished");

    pg_conn
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Инициализируем менеджер логирования
    let _log_guard = initialize_logs();

    // Базовый span для логирования
    let span = debug_span!("root_span");
    let _span_guard = span.enter();

    // Создаем объект базы данных
    let database = web::Data::new(open_database().await);

    // Система для хеширования паролей
    let token =  web::Data::new(TokenService::new("test_secret_key".to_string())); // TODO: Ключ из окружения

    HttpServer::new(move ||{
            // Приложение создается для каждого потока свое собственное
            App::new()
                .wrap(TracingLogger)
                .app_data(database.clone())
                .app_data(token.clone())
                .configure(configure_routes)
        }) 
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests{
    use super::*;
    use serde_json::{
        json
    };
    use actix_web::{
        test, 
        web, 
        App
    };

    #[actix_rt::test]
    async fn test_server() {
        std::env::set_var("DATABASE_URL", "postgres://actix:actix@localhost:5432/actix");

        // Создаем объект базы данных
        let database = web::Data::new(open_database().await);

        // Система для хеширования паролей
        let token =  web::Data::new(TokenService::new("test_secret_key".to_string())); // TODO: Ключ из окружения

        // Тестовое приложение
        let mut app = test::init_service(App::new()
                .app_data(database.clone())
                .app_data(token.clone())
                .configure(configure_routes))
            .await;
        
        // Валидный запрос логина на сервер
        let signup_req = test::TestRequest::post()
            .uri("/signup")
            .set_json(&json!({
                "user_login": "test_user",
                "email": "valid@email.com",
                "password": "valid_password"
            }))
            .to_request();
        
        // Ответ на запрос
        let signup_resp: models::user::UserData = test::read_response_json(&mut app, signup_req)
            .await;
            
        println!("Signup valid response: {:#?}", signup_resp);
    }
}
