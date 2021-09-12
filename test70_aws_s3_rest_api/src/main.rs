use eyre::{Context, ContextCompat, WrapErr};
use futures::{Stream, StreamExt};
use http_body::Body as BodyTrait;
use hyper::{
    body::{Body as BodyStruct, HttpBody, aggregate, to_bytes},
    client::HttpConnector,
    http::{header, Version},
    Client, Method, Request, Response, StatusCode, Uri,
};
use hyper_rustls::HttpsConnector;
use mime::Mime;
use std::{convert::TryFrom, process::exit, str::FromStr, time::Duration};
use tokio::time::timeout;
use tracing::{debug, error, info};
use tracing_error::ErrorLayer;
use tracing_log::LogTracer;
use tracing_subscriber::prelude::*;

///////////////////////////////////////////////////////////////////////////////////////////

fn initialize_logs() -> Result<(), eyre::Error> {
    // Фильтрация на основе настроек
    let filter = tracing_subscriber::EnvFilter::from_default_env();

    // Логи в stdout
    let stdoud_sub = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

    // Error layer для формирования слоя ошибки
    let error_layer = ErrorLayer::default();

    // Суммарный обработчик
    let full_subscriber = tracing_subscriber::registry().with(filter).with(error_layer).with(stdoud_sub);

    // Враппер для библиотеки log
    LogTracer::init().wrap_err("Log wrapper create failed")?;

    // Установка по-умолчанию
    tracing::subscriber::set_global_default(full_subscriber).wrap_err("Global subscriber set failed")?;

    Ok(())
}

async fn execute_app() -> Result<(), eyre::Error> {
    // Нужн для резолва DNS адресов на пуле потоков (Только HTTP соединения)
    /*let http_connector = {
        let mut http_connector = HttpConnector::new();
        http_connector.set_nodelay(false);
        http_connector.set_reuse_address(false);
        http_connector
    };*/

    // Коннектор для работы уже с HTTPS
    let https_connector = HttpsConnector::with_native_roots();

    // Клиент с коннектором
    let http_client = Client::builder().build::<_, BodyStruct>(https_connector);

    // Адрес запроса
    let uri = Uri::builder()
        .scheme("https")
        .authority("httpbin.org")
        .path_and_query("/ip")
        .build()
        .wrap_err("Uri build failed")?;
    info!(?uri, "Uri");

    // Объект запроса
    let request = Request::builder()
        .version(Version::HTTP_11)
        .uri(uri)
        .header(header::HOST, "httpbin.org")
        .method(Method::GET)
        .body(BodyStruct::empty())
        .wrap_err("Request build error")?;
    info!(?request, "Request");

    // Объект ответа
    let response = http_client.request(request).await.wrap_err("Http response error")?;
    info!(?response, "Response");

    // Получаем длину контента
    let content_length: usize = response
        .headers()
        .get(header::CONTENT_LENGTH)
        .ok_or_else(|| eyre::eyre!("Content length header is missing"))?
        .to_str()?
        .parse()
        .wrap_err("Content Length parse failed")?;
    info!(content_length, "Content length");

    // Получаем тип контента
    let content_type_mime: mime::Mime = response
        .headers()
        .get(header::CONTENT_TYPE)
        .ok_or_else(|| eyre::eyre!("Content type header is missing"))?
        .to_str()
        .wrap_err("Content type header to string convert failed")?
        .parse()
        .wrap_err("Mime parse failed")?;
    info!(?content_type_mime, "Content type");

    // Работаем с ответом
    if content_type_mime == mime::APPLICATION_JSON {
        let mut body = response.into_body();

        match timeout(Duration::from_secs(1), body.trailers()).await {
            Ok(trailers) => {
                let trailers = trailers.wrap_err("Trailers receive error")?;
                info!(?trailers, "Body trailers");
            }
            Err(_) => {
                info!("Body trailers timeout");
            }
        }

        let body_data = to_bytes(body).await.wrap_err("Body data receive")?;
        info!(?body_data, "Body data");

        // let body_data = aggregate(body).await.wrap_err("Body data receive")?;
        // info!(?body_data, "Body data");

        // while let Some(body_data) = body.data().await{
        //     let body_data = body_data?;
        //     info!(?body_data, "Body data");
        // }

        // while let Some(data) = body.next().await {
        //     info!(?data, "Body chunk");
        // }
    } else if content_type_mime == mime::TEXT_PLAIN {
    } else {
        eyre::eyre!("Unknown content type");
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // Бектрейсы в ошибках
    color_eyre::install().unwrap();

    // Читаем переменные окружения из файлика
    dotenv::from_filename("env/test.env").unwrap();

    // Логи
    initialize_logs().unwrap();

    if let Err(err) = execute_app().await {
        error!("{:?}", err);
        exit(1);
    } else {
        exit(0);
    }
}
