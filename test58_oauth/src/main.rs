mod error;
mod auth_handlers;
mod app_middlewares;
mod constants;
mod responses;
mod database;
mod env_app_params;
mod helpers;

use actix_files::{
    Files
};
use actix_web::{App, 
    FromRequest, 
    HttpServer, 
    guard::{
        self
    }, 
    web::{
        self, 
        Data
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
    Identity, 
    IdentityService, 
    RequestIdentity
};
use tracing::{
    Instrument, 
    Value,
    debug_span, 
    error_span, 
    info_span, 
    trace_span,
    event, 
    instrument, 
    debug,
    error,
    info,
    trace
};
use tracing_subscriber::{
    prelude::{
        *
    }
};
use tracing_actix_web::{
    TracingLogger
};
use tracing_opentelemetry::{
    OpenTelemetrySpanExt,
    PreSampledTracer
};
use crate::{
    error::{
        AppError
    },
    env_app_params::{
        FacebookEnvParams,
        GoogleEnvParams
    },
    app_middlewares::{
        create_error_middleware,
        // create_check_login_middleware
    },
    database::{
        Database,
        UserInfo
    },
    helpers::{
        get_full_user_info_for_identity,
        get_uuid_from_ident_with_db_check
    }
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// #[instrument]
async fn index(handlebars: web::Data<Handlebars<'_>>, 
               id: Identity,
               db: web::Data<Database>) -> Result<web::HttpResponse, AppError> {
    let span = debug_span!("index_page_span", "user" = tracing::field::Empty);
    let _enter_guard = span.enter();

    // Проверка идентификатора пользователя
    // TODO: приходится делать это здесь, а не в middleware, так как
    // есть проблемы с асинхронным запросом к базе в middleware 
    let full_info = match get_full_user_info_for_identity(&id, &db).await? {
        Some(full_info) => full_info,
        None => {
            debug!("Redirect code from handler");
            // Возвращаем код 302 и Location в заголовках для перехода
            return Ok(web::HttpResponse::Found()
                        .header(actix_web::http::header::LOCATION, constants::LOGIN_PATH)
                        .finish())
        }
    };

    span.record("user", &tracing::field::debug(&full_info));

    let template_data = serde_json::json!({
        "uuid": full_info.user_uuid,
        "facebook_uid": full_info.facebook_uid,
        "google_uid": full_info.google_uid,
    });

    // Рендерим шаблон
    let body = handlebars.render(constants::INDEX_TEMPLATE, &template_data)?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// #[instrument]
async fn login_page(handlebars: web::Data<Handlebars<'_>>,
                    id: Identity,
                    db: web::Data<Database>) -> Result<web::HttpResponse, AppError> {
    let span = debug_span!("login_page_span", "user" = tracing::field::Empty);
    let _enter_guard = span.enter();

    // Проверка идентификатора пользователя
    // TODO: приходится делать это здесь, а не в middleware, так как
    // есть проблемы с асинхронным запросом к базе в middleware 
    if get_uuid_from_ident_with_db_check(&id, &db).await?.is_some() {
        debug!("Redirect code from handler");
        // Возвращаем код 302 и Location в заголовках для перехода
        return Ok(web::HttpResponse::Found()
                    .header(actix_web::http::header::LOCATION, constants::INDEX_PATH)
                    .finish())
    }

    let body = handlebars.render(constants::LOGIN_TEMPLATE, &serde_json::json!({}))?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// #[instrument]
async fn logout(id: Identity) -> Result<web::HttpResponse, AppError> {
    id.forget();

    // Возвращаем код 302 и Location в заголовках для перехода
    return Ok(web::HttpResponse::Found()
                .header(actix_web::http::header::LOCATION, constants::LOGIN_PATH)
                .finish())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Создаем менеджер шаблонов и регистрируем туда нужные
#[instrument]
fn create_templates<'a>() -> Handlebars<'a> {
    let span = debug_span!("Template engine configure", templates_type = "handlebars");
    let _g = span.enter();

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

/// Функция непосредственного конфигурирования приложения
/// Для каждого потока исполнения будет создано свое приложение
fn configure_new_app(config: &mut web::ServiceConfig) {
    let span = debug_span!("Server application configure");
    let _g = span.enter();

    config
        .service(web::resource(constants::INDEX_PATH)
                    .route(web::route()
                            .guard(guard::Get())
                            .to(index)))
        .service(web::resource(constants::LOGIN_PATH)
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
        .service(Files::new("static/js", "static/js"));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Инициализируем менеджер логирования
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
        .with(tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::TRACE.into()))
        .with(file_sub)
        .with(opentelemetry_sub)
        .with(tracing_subscriber::EnvFilter::from_default_env()) // Фильтр стоит специально в этом месте, чтобы фильтровать лишь сообщения для терминала
        .with(stdoud_sub);
    tracing::subscriber::set_global_default(full_subscriber).unwrap();

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

            // TODO: Session middleware

            // Приложение создается для каждого потока свое собственное
            // Порядок выполнения Middleware обратный, снизу вверх
            App::new()
                .wrap(identity_middleware)
                .wrap(create_error_middleware())
                .wrap(TracingLogger)
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
