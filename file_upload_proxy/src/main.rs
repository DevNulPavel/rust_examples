mod app_arguments;
mod auth_token_provider;
mod error;
mod helpers;
mod oauth2;
mod types;

use crate::{
    app_arguments::AppArguments,
    auth_token_provider::AuthTokenProvider,
    error::{ErrorWithStatusAndDesc, WrapErrorWithStatusAndDesc},
    types::HttpClient,
};
use eyre::WrapErr;
use futures::future::pending;
use hyper::{
    body::{to_bytes, Body as BodyStruct, Bytes},
    http::{header, method::Method, status::StatusCode},
    server::{conn::AddrStream, Server},
    service::{make_service_fn, service_fn},
    Client, Request, Response,
};
use hyper_rustls::HttpsConnector;
use std::{convert::Infallible, net::SocketAddr, process::exit, sync::Arc};
use structopt::StructOpt;
use tracing::{debug, error, info, instrument, trace, warn};
use tracing_futures::Instrument;

fn initialize_logs(arguments: &AppArguments) -> Result<(), eyre::Error> {
    use tracing_subscriber::prelude::*;

    let level = match arguments.verbose {
        0 => tracing::Level::ERROR,
        1 => tracing::Level::WARN,
        2 => tracing::Level::INFO,
        3 => tracing::Level::DEBUG,
        4 => tracing::Level::TRACE,
        _ => {
            panic!("Verbose level must be in [0, 4] range");
        }
    };

    // Фильтрация на основе настроек
    let filter = tracing_subscriber::filter::LevelFilter::from_level(level);

    // Логи в stdout
    let stdoud_sub = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

    // Error layer для формирования слоя ошибки
    let error_layer = tracing_error::ErrorLayer::default();

    // Суммарный обработчик
    let full_subscriber = tracing_subscriber::registry().with(filter).with(error_layer).with(stdoud_sub);

    // Враппер для библиотеки log
    tracing_log::LogTracer::init().wrap_err("Log wrapper create failed")?;

    // Установка по-умолчанию
    tracing::subscriber::set_global_default(full_subscriber).wrap_err("Global subscriber set failed")?;

    Ok(())
}

fn response_with_status_and_empty_body(status: StatusCode) -> Response<BodyStruct> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_LENGTH, 0)        
        .body(BodyStruct::empty())
        .expect("Static fail response create failed") // Статически создаем ответ, здесь не критично
}

fn response_with_status_and_error(status: StatusCode, err_desc: &str) -> Response<BodyStruct> {
    let error_json = format!(r#"{{"description": "{}"}}"#, err_desc);
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.essence_str()) // TODO: Check
        .header(header::CONTENT_LENGTH, error_json.as_bytes().len())
        .body(BodyStruct::from(error_json))
        .expect("Static fail response create failed") // Статически создаем ответ, здесь не критично
}

fn response_with_status_desc_and_trace_id(status: StatusCode, err_desc: &str, trace_id: &str) -> Response<BodyStruct> {
    let error_json = format!(r#"{{"error_trace_id": "{}", "desc": "{}"}}"#, trace_id, err_desc);
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.essence_str()) // TODO: Check
        .header(header::CONTENT_LENGTH, error_json.as_bytes().len())
        .body(BodyStruct::from(error_json))
        .expect("Static fail response create failed") // Статически создаем ответ, здесь не критично
}

// Трассировка настраивается уровнем выше
// #[instrument(level = "error")]
async fn service_handler(req: Request<BodyStruct>, token_provider: &AuthTokenProvider) -> Result<Response<BodyStruct>, ErrorWithStatusAndDesc> {
    // debug!("Request processing begin");

    match (req.method(), req.uri().path()) {
        // Запрос к корню
        (&Method::GET, "/") => {
            info!("Root");
            let response = Response::builder()
                .status(StatusCode::MOVED_PERMANENTLY)
                .header(header::LOCATION, "/help")
                .body(BodyStruct::empty())
                .wrap_err_with_500()?;

            Ok(response)
        }

        // Помощь
        (&Method::GET, "/help") => {
            info!("Help");
            let response = Response::builder()
                .status(StatusCode::OK)
                .body(BodyStruct::from("Try to send POST request at '/echo'"))
                .wrap_err_with_500()?;
            Ok(response)
        }

        // Отладочным образом получаем токен
        (&Method::GET, "/token") => {
            info!("Token");

            let token = token_provider.get_token().await.wrap_err_with_status_desc(StatusCode::UNAUTHORIZED, "Google cloud token receive failed".into())?;

            let json_text = format!(r#"{{"token": "{}"}}"#, token);

            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.essence_str()) // TODO: Check
                .header(header::CONTENT_LENGTH, json_text.as_bytes().len())                
                .body(BodyStruct::from(json_text))
                .wrap_err_with_500()?;
            Ok(response)
        }

        // Любой другой запрос
        _ => {
            warn!("Invalid request");
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(BodyStruct::empty())
                .wrap_err_with_500()?;
            Ok(response)
        }
    }
}

async fn run_server(app_arguments: AppArguments, token_provider: AuthTokenProvider) -> Result<(), eyre::Error> {
    // Перемещаем в кучу для свободного доступа из разных обработчиков
    let token_provider = Arc::new(token_provider);

    // Сервис необходим для каждого соединения, поэтому создаем враппер, который будет генерировать наш сервис
    let make_svc = make_service_fn(move |conn: &AddrStream| {
        let token_provider = token_provider.clone();

        // Получаем адрес удаленного подключения
        let remote_addr = conn.remote_addr();
        async move {
            // Создаем сервис из функции с помощью service_fn
            Ok::<_, Infallible>(service_fn(move |req| {
                let token_provider = token_provider.clone();

                async move {
                    // Создаем идентификатор трассировки для отслеживания ошибок в общих логах
                    let trace_id = format!("{:x}", uuid::Uuid::new_v4());

                    // Создаем span с идентификатором трассировки
                    let span = tracing::error_span!("request", 
                        remote_ip = %remote_addr, 
                        trace_id = %trace_id,
                        path = req.uri().path());

                    // Обработка сервиса
                    match service_handler(req, &token_provider).instrument(span).await {
                        resp @ Ok(_) => resp,
                        Err(err) => {
                            // Выводим ошибку в консоль
                            error!("{}", err);

                            // Ответ в виде ошибки
                            let resp = response_with_status_desc_and_trace_id(err.status, &err.desc, &trace_id);

                            Ok(resp)
                        }
                    }
                }
            }))
        }
    });

    // Адрес
    let addr = SocketAddr::from(([0, 0, 0, 0], app_arguments.port));

    // Создаем сервер c ожиданием завершения работы
    Server::bind(&addr)
        .serve(make_svc)
        .with_graceful_shutdown(async {
            // Wait for the CTRL+C signal
            if let Err(err) = tokio::signal::ctrl_c().await {
                warn!("Shutdown signal awaiter setup failed, continue without: {}", err);
                // Создаем поэтому вечную future
                pending::<()>().await;
            }
            info!("Shutdown signal received");
        })
        .await
        .wrap_err("Server awaiting fail")?;

    Ok(())
}

/// Выполняем валидацию переданных аргументов приложения
fn validate_arguments(arguments: &AppArguments) -> Result<(), &str> {
    macro_rules! validate_argument {
        ($argument: expr, $desc: literal) => {
            if $argument == false {
                return Err($desc);
            }
        };
    }

    validate_argument!(arguments.google_credentials_file.exists(), "Google credential file does not exist");
    validate_argument!(arguments.google_credentials_file.is_file(), "Google credential file is not a file");
    Ok(())
}

fn build_http_client() -> HttpClient {
    // Коннектор для работы уже с HTTPS
    let https_connector = HttpsConnector::with_native_roots();

    // Клиент с коннектором
    let http_client = std::sync::Arc::new(Client::builder().set_host(false).build::<_, BodyStruct>(https_connector));

    http_client
}

fn main() {
    // Бектрейсы в ошибках
    color_eyre::install().expect("Color eyre initialize failed");

    // Аргументы приложения
    let app_arguments = AppArguments::from_args();
    debug!("App arguments: {:?}", app_arguments);

    // Проверка аргументов приложения
    if let Err(err_desc) = validate_arguments(&app_arguments) {
        eprintln!("Invalid argument: {}", err_desc);
        exit(1);
    }

    // Логи
    initialize_logs(&app_arguments).expect("Logs init");

    // Клиент для https
    let http_client = build_http_client();

    // Создаем провайдер для токенов
    let token_provider = AuthTokenProvider::new(
        http_client.clone(),
        &app_arguments.google_credentials_file,
        "https://www.googleapis.com/auth/devstorage.read_write",
    )
    .expect("Token provider create failed");

    // Создаем рантайм для работы сервера
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Tokio runtime build");

    // Стартуем сервер
    runtime.block_on(run_server(app_arguments, token_provider)).expect("Server running fail");
}
