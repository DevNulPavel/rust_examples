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
    helpers::{response_with_status_desc_and_trace_id, get_content_length},
};
use serde::Deserialize;
use serde_json::from_reader as json_from_reader;
use eyre::WrapErr;
use futures::future::pending;
use hyper::{
    body::{to_bytes, aggregate, Body as BodyStruct, Bytes, Buf},
    http::{header, uri::{Uri, Authority, PathAndQuery}, method::Method, status::StatusCode},
    server::{conn::AddrStream, Server},
    service::{make_service_fn, service_fn},
    Client, Request, Response,
};
use hyper_rustls::HttpsConnector;
use std::{convert::Infallible, net::SocketAddr, process::exit, sync::Arc};
use structopt::StructOpt;
use tracing::{debug, error, info, instrument, trace, warn};
use tracing_futures::Instrument;

struct App{
    app_arguments: AppArguments, 
    http_client: HttpClient, 
    token_provider: AuthTokenProvider
}

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

// Трассировка настраивается уровнем выше
// #[instrument(level = "error")]
async fn service_handler(app: &App, req: Request<BodyStruct>) -> Result<Response<BodyStruct>, ErrorWithStatusAndDesc> {
    // debug!("Request processing begin");

    match (req.method(), req.uri().path()) {
        // Отладочным образом получаем токен
        (&Method::GET, "/token") => {
            info!("Token");

            let token = app.token_provider.get_token().await.wrap_err_with_status_desc(StatusCode::UNAUTHORIZED, "Google cloud token receive failed".into())?;

            let json_text = format!(r#"{{"token": "{}"}}"#, token);

            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.essence_str()) // TODO: Check
                // .header(header::CONTENT_LENGTH, json_text.as_bytes().len())                
                .body(BodyStruct::from(json_text))
                .wrap_err_with_500()?;
            Ok(response)
        }

        // Выгружаем данные в Cloud
        (&Method::POST, "/upload_file") => {
            info!("File uploading");

            // Получаем размер данных
            let data_length = get_content_length(req.headers())
                .wrap_err_with_status_desc(StatusCode::LENGTH_REQUIRED, "Content-Length header parsing failed".into())?
                .wrap_err_with_status_desc(StatusCode::LENGTH_REQUIRED, "Content-Length header is missing".into())?;

            // TODO: Получаем токен из запроса и проверяем

            // Получаем токен
            let token = app.token_provider.get_token().await.wrap_err_with_status_desc(StatusCode::UNAUTHORIZED, "Google cloud token receive failed".into())?;

            // Имя нашего файлика
            let file_name = format!("{:x}.txt", uuid::Uuid::new_v4());

            // Адрес запроса
            let uri = Uri::builder()
                .scheme("https")
                .authority(Authority::from_static("storage.googleapis.com"))
                .path_and_query(format!(
                    "/upload/storage/v1/b/{}/o?name={}&uploadType=media&fields={}",
                    urlencoding::encode(&app.app_arguments.google_bucket_name),
                    urlencoding::encode(&file_name),
                    urlencoding::encode("md5Hash,mediaLink") // Только нужные поля в ответе сервера
                ))
                .build()
                .wrap_err_with_500()?;
            debug!("Request uri: {}", uri);

            // Объект запроса
            let request = Request::builder()
                .method(Method::POST)
                .version(hyper::Version::HTTP_2)
                .uri(uri)
                // TODO: Что-то не так с установкой значения host, если выставить, то фейлится запрос
                // Может быть дело в регистре?
                // .header(header::HOST, "oauth2.googleapis.com")
                .header(header::USER_AGENT, "hyper")
                .header(header::CONTENT_LENGTH, data_length)
                .header(header::ACCEPT, mime::APPLICATION_JSON.to_string()) // TODO: Optimize
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, mime::OCTET_STREAM.to_string()) // TODO: Optimize
                .body(BodyStruct::wrap_stream(req.into_body()))
                .wrap_err_with_500()?;
            debug!("Request object: {:?}", request);
            
            // Объект ответа
            let response = app.http_client.request(request).await.wrap_err_with_status_desc(StatusCode::INTERNAL_SERVER_ERROR, "Google cloud error".into())?;
            debug!("Google response: {:?}", response);

            // Статус
            let status = response.status();
            debug!(?status);

            // Обрабатываем в зависимости от ответа
            if status.is_success() {
                #[derive(Debug, Deserialize)]
                struct Info {
                    #[serde(rename = "md5Hash")]
                    md5: String,
                    #[serde(rename = "mediaLink")]
                    link: String,
                }
                // Данные
                let body_data = aggregate(response).await.wrap_err_with_status_desc(StatusCode::INTERNAL_SERVER_ERROR, "Google cloud response receive failed".into())?;
                let info = json_from_reader::<_, Info>(body_data.reader()).wrap_err_with_status_desc(StatusCode::INTERNAL_SERVER_ERROR, "Google cloud response parsing failed".into())?;
                debug!("Uploading result: {:?}", info);

                let json_text = format!(r#"{{"link": "{}"}}"#, info.link);

                let response = Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.essence_str()) // TODO: Check
                    .header(header::CONTENT_LENGTH, json_text.as_bytes().len())                
                    .body(BodyStruct::from(json_text))
                    .wrap_err_with_500()?;

                Ok(response)
            } else {
                // Данные
                let body_data = to_bytes(response).await.wrap_err_with_status_desc(StatusCode::INTERNAL_SERVER_ERROR, "Google cloud response receive failed".into())?;
                error!("Upload fail result: {:?}", body_data);

                match std::str::from_utf8(&body_data).ok(){
                    Some(text) => {
                        error!("Upload fail result text: {}", text);
                        let resp = format!("Google error response: {}", text);
                        return Err(ErrorWithStatusAndDesc::new_with_status_desc(StatusCode::INTERNAL_SERVER_ERROR, resp.into()));
                    },
                    None => {
                        return Err(ErrorWithStatusAndDesc::new_with_status_desc(StatusCode::INTERNAL_SERVER_ERROR, "Google uploading failed".into()));
                    }
                }
            }
        }

        // Любой другой запрос
        _ => {
            error!("Invalid request");
            return Err(ErrorWithStatusAndDesc::new_with_status_desc(StatusCode::BAD_REQUEST, "Wrong path".into()));
        }
    }
}

async fn run_server(app: App) -> Result<(), eyre::Error> {
    // Перемещаем в кучу для свободного доступа из разных обработчиков
    let app = Arc::new(app);

    // Адрес
    let addr = SocketAddr::from(([0, 0, 0, 0], app.app_arguments.port));

    // Сервис необходим для каждого соединения, поэтому создаем враппер, который будет генерировать наш сервис
    let make_svc = make_service_fn(move |conn: &AddrStream| {
        let app = app.clone();

        // Получаем адрес удаленного подключения
        let remote_addr = conn.remote_addr();
        async move {
            // Создаем сервис из функции с помощью service_fn
            Ok::<_, Infallible>(service_fn(move |req| {
                let app = app.clone();

                async move {
                    // Создаем идентификатор трассировки для отслеживания ошибок в общих логах
                    let trace_id = format!("{:x}", uuid::Uuid::new_v4());

                    // Создаем span с идентификатором трассировки
                    let span = tracing::error_span!("request", 
                        remote_ip = %remote_addr, 
                        trace_id = %trace_id,
                        path = req.uri().path());

                    // Обработка сервиса
                    match service_handler(&app, req).instrument(span).await {
                        resp @ Ok(_) => resp,
                        Err(err) => {
                            // Выводим ошибку в консоль
                            eprintln!("{}", err);

                            // Ответ в виде ошибки
                            let resp = response_with_status_desc_and_trace_id(err.status, &err.desc, &trace_id);

                            Ok(resp)
                        }
                    }
                }
            }))
        }
    });

    // Создаем сервер c ожиданием завершения работы
    Server::bind(&addr)
        .serve(make_svc)
        /*.with_graceful_shutdown(async {
            // https://github.com/hyperium/hyper/issues/1681
            // https://github.com/hyperium/hyper/issues/1668
            // Есть проблема с одновременным использованием клиента и сервера
            // Gracefull Shutdown сервера работает долго очень
            // Вроде как нужно просто уничтожать все объекты HTTP клиента заранее

            // Wait for the CTRL+C signal
            if let Err(err) = tokio::signal::ctrl_c().await {
                warn!("Shutdown signal awaiter setup failed, continue without: {}", err);
                // Создаем поэтому вечную future
                pending::<()>().await;
            }
            println!("Shutdown signal received, please wait all timeouts");
        })*/
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
    validate_argument!(!arguments.google_bucket_name.is_empty(), "Target Google bucket can't be empty");
    Ok(())
}

fn build_http_client() -> HttpClient {
    // Коннектор для работы уже с HTTPS
    let https_connector = HttpsConnector::with_native_roots();

    // Клиент с коннектором
    let http_client = Client::builder().set_host(false).build::<_, BodyStruct>(https_connector);

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

    // Контейнер со всеми менеджерами
    let app = App{
        app_arguments,
        http_client,
        token_provider
    };

    // Стартуем сервер
    runtime.block_on(run_server(app)).expect("Server running fail");
}
