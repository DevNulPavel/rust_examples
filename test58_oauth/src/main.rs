mod error;
mod app_middlewares;
mod constants;
mod responses;
mod database;
mod app_params;
mod helpers;
mod handlers;

use actix_web::{
    App, 
    HttpServer,
    web::{
        self
    }
};
use handlebars::{
    Handlebars
};
use rand::{
    Rng
};
use actix_identity::{
    CookieIdentityPolicy, 
    IdentityService
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
    app_params::{
        FacebookEnvParams,
        GoogleEnvParams,
        AppEnvParams
    },
    database::{
        Database
    },
    handlers::{
        retup_routes
    },

};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

struct LogGuards{
    _file_appender_guard: tracing_appender::non_blocking::WorkerGuard,
    _opentelemetry_uninstall: opentelemetry_jaeger::Uninstall
}

fn initialize_logs() -> LogGuards{
    // Логирование в файлики
    let (non_blocking_appender, _file_appender_guard) = 
        tracing_appender::non_blocking(tracing_appender::rolling::hourly("logs", "app_log"));
    let file_sub = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .json()
        .with_writer(non_blocking_appender);

    // Логи в stdout
    let stdoud_sub = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_thread_ids(true)
        .with_ansi(true)
        .with_writer(std::io::stdout);

    // Логи opentelemetry
    let (opentelemetry_tracer, _opentelemetry_uninstall) = opentelemetry_jaeger::new_pipeline()
        .with_service_name("oauth_server")
        .install()
        .unwrap();
    /*let (opentelemetry_tracer, _un) = opentelemetry::sdk::export::trace::stdout::new_pipeline()
        .install();*/
    let opentelemetry_sub = tracing_opentelemetry::layer()
        .with_tracer(opentelemetry_tracer);

    // Суммарный обработчик
    let full_subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::default()
                .add_directive(tracing::Level::TRACE.into())
                .and_then(file_sub)
                .and_then(opentelemetry_sub))
        .with(tracing_subscriber::EnvFilter::from_default_env() // TODO: Почему-то все равно не работает
                .and_then(stdoud_sub));

    // Установка по-умолчанию
    tracing::subscriber::set_global_default(full_subscriber).unwrap();

    LogGuards{
        _file_appender_guard,
        _opentelemetry_uninstall
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Создаем менеджер шаблонов и регистрируем туда нужные
fn create_templates<'a>() -> Handlebars<'a> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_file(constants::INDEX_TEMPLATE, "templates/index.hbs")
        .unwrap();
    handlebars
        .register_template_file(constants::LOGIN_TEMPLATE, "templates/login.hbs")
        .unwrap();  
    
    handlebars
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Вычитывем переменные окружения из файлика .env и добавляем их в окружение
    dotenv::dotenv().ok();

    // Инициализируем менеджер логирования
    let _log_guard = initialize_logs();

    // Базовый span для логирования
    let span = debug_span!("root_span");
    let _span_guard = span.enter();

    // Получаем параметры приложения
    let app_env_params = web::Data::new(AppEnvParams::get_from_env());
    let facebook_env_params = web::Data::new(FacebookEnvParams::get_from_env());
    let google_env_params = web::Data::new(GoogleEnvParams::get_from_env());

    // Создаем подключение к нашей базе
    let sqlite_conn = web::Data::new(Database::open().await);

    // Создаем шареную ссылку на обработчик шаблонов
    // Пример работы с шаблонами
    // https://github.com/actix/examples/tree/master/template_engines/handlebars
    let handlebars = web::Data::new(create_templates());

    // Ключ для шифрования кук, генерируется каждый раз при запуске сервера
    let private_key = rand::thread_rng().gen::<[u8; 32]>();

    // Создаем общего http клиента для разных запросов
    let http_client = web::Data::new(reqwest::Client::new());

    // Фактический адрес сервера
    let server_address = format!("0.0.0.0:{}", app_env_params.http_port);

    HttpServer::new(move ||{
            // Настраиваем middleware идентификации пользователя, делает зашифрованную куку у пользователя в браузере,
            // тем самым давая возможность проверять залогинен ли пользователь или нет
            let identity_middleware = {
                let policy = CookieIdentityPolicy::new(&private_key)
                    .name("auth-logic")
                    .max_age(60 * 60 * 24 * 30) // 30 дней максимум
                    .http_only(true)
                    .secure(false);
                IdentityService::new(policy)
            };

            // Приложение создается для каждого потока свое собственное
            // Порядок выполнения Middleware обратный, снизу вверх
            App::new()
                .wrap(identity_middleware)
                .wrap(TracingLogger)
                .app_data(sqlite_conn.clone())
                .app_data(handlebars.clone())
                .app_data(app_env_params.clone())
                .app_data(facebook_env_params.clone())
                .app_data(google_env_params.clone())
                .app_data(http_client.clone())
                .configure(retup_routes)
        })
        .bind(server_address)?
        .run()
        .await
}
