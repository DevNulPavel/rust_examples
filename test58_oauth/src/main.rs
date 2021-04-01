mod error;
mod auth_handlers;
mod app_middlewares;
mod constants;
mod responses;
mod database;
mod env_app_params;
mod helpers;
mod http_handlers;

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
    env_app_params::{
        FacebookEnvParams,
        GoogleEnvParams
    },
    app_middlewares::{
        create_error_middleware,
        create_user_info_middleware,
        create_auth_check_middleware
    },
    database::{
        Database
    },
    http_handlers::{
        index,
        login_page,
        logout
    }
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

struct LogGuards{
    _file_appender_guard: tracing_appender::non_blocking::WorkerGuard,
    _opentelemetry_uninstall: opentelemetry_jaeger::Uninstall
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
    let (opentelemetry_tracer, _opentelemetry_uninstall) = opentelemetry_jaeger::new_pipeline()
        .with_service_name("oauth_server")
        .install()
        .unwrap();
    /*let (opentelemetry_tracer, _un) = opentelemetry::sdk::export::trace::stdout::new_pipeline()
        .install();*/
    let opentelemetry_sub = tracing_opentelemetry::layer()
        .with_tracer(opentelemetry_tracer);
    let full_subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::default()
                .add_directive(tracing::Level::TRACE.into())
                .and_then(file_sub)
                .and_then(opentelemetry_sub))
        .with(tracing_subscriber::EnvFilter::from_default_env() // TODO: Почему-то все равно не работает
                .and_then(stdoud_sub));
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
        .register_template_file(constants::ERROR_TEMPLATE, "templates/error.hbs")
        .unwrap();   
    
    handlebars
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Функция непосредственного конфигурирования приложения
/// Для каждого потока исполнения будет создано свое приложение
fn configure_new_app(config: &mut web::ServiceConfig) {
    config
        .service(web::resource(constants::INDEX_PATH)
                    .wrap(create_user_info_middleware(
                            || {
                                web::HttpResponse::Found()
                                    .header(actix_web::http::header::LOCATION, constants::LOGIN_PATH)
                                    .finish()
                            }))
                    .route(web::route()
                            .guard(guard::Get())
                            .to(index)))
        .service(web::resource(constants::LOGIN_PATH)
                    .wrap(create_auth_check_middleware(
                            false,
                            || {
                                web::HttpResponse::Found()
                                    .header(actix_web::http::header::LOCATION, constants::INDEX_PATH)
                                    .finish()
                            }))
                    .route(web::route()
                                .guard(guard::Get())
                                .to(login_page)))                         
        .service(web::resource(constants::LOGOUT_PATH)
                    .route(web::route()
                                .guard(guard::Post())
                                .to(logout))) 
        .service(web::scope(constants::FACEBOOK_SCOPE_PATH)
                    .service(web::resource(constants::LOGIN_PATH)
                                .route(web::route()
                                        .guard(guard::Post())
                                        .to(auth_handlers::login_with_facebook)))
                    .service(web::resource(constants::AUTH_CALLBACK_PATH)
                                .route(web::route()
                                        .guard(guard::Get())
                                        .to(auth_handlers::facebook_auth_callback))))
        .service(web::scope(constants::GOOGLE_SCOPE_PATH)
                    .service(web::resource(constants::LOGIN_PATH)
                                .route(web::route()
                                        .guard(guard::Post())
                                        .to(auth_handlers::login_with_google)))
                    .service(web::resource(constants::AUTH_CALLBACK_PATH)
                                .route(web::route()
                                        .guard(guard::Get())
                                        .to(auth_handlers::google_auth_callback))))
        .service(Files::new("static/css", "static/css"))
        .service(Files::new("static/js", "static/js"))
        .service(Files::new("static/images", "static/images"));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Инициализируем менеджер логирования
    let _log_guard = initialize_logs();

    // Базовый span для логирования
    let span = debug_span!("root_span");
    let _span_guard = span.enter();

    // Получаем параметры Facebook + Google
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

            // Специальный middleware для возможности работы iframe
            /*let cors_mid = actix_cors::Cors::default()
                .allowed_origin("http://localhost:9999")
                .allowed_origin("http://localhost:8080");*/

            // Приложение создается для каждого потока свое собственное
            // Порядок выполнения Middleware обратный, снизу вверх
            App::new()
                .wrap(identity_middleware)
                .wrap(create_error_middleware())
                .wrap(TracingLogger)
                // .wrap(cors_mid)
                .app_data(sqlite_conn.clone())
                .app_data(handlebars.clone())
                .app_data(facebook_env_params.clone())
                .app_data(google_env_params.clone())
                .app_data(http_client.clone())
                .configure(configure_new_app)
        }) 
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
