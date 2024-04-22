mod helpers;
mod oauth2;
mod types;

use crate::{
    oauth2::{get_token_data, ServiceAccountData},
    types::HttpClient,
};
use eyre::WrapErr;
use hyper::{
    body::{to_bytes, aggregate, Body as BodyStruct, Buf},
    header,
    http::uri::{Authority, Uri},
    Client, Method, Request,
};
use hyper_rustls::HttpsConnector;
use serde::Deserialize;
use serde_json::from_reader as json_from_reader;
use std::{convert::From, path::Path, process::exit};
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

fn build_http_client() -> HttpClient {
    // Коннектор для работы уже с HTTPS
    let https_connector = HttpsConnector::with_native_roots();

    // Клиент с коннектором
    let http_client = Client::builder().set_host(false).build::<_, BodyStruct>(https_connector);

    http_client
}

async fn execute_app() -> Result<(), eyre::Error> {
    // Описание логики получения токена из сервисного аккаунта
    // https://developers.google.com/identity/protocols/oauth2/service-account#httprest
    // https://cloud.google.com/storage/docs/authentication

    // Данные по сервисному аккаунту
    let service_acc_data =
        ServiceAccountData::new_from_file(Path::new("./env/test_google_service_account.json")).wrap_err("Service account read")?;
    debug!("Service account data: {:?}", service_acc_data);

    // Клиент для http
    let http_client = build_http_client();

    // Получаем токен для работы
    let token_data = get_token_data(
        &http_client,
        &service_acc_data,
        "https://www.googleapis.com/auth/devstorage.read_write",
    )
    .await
    .wrap_err("Token receive")?;
    info!("Received token: {:?}", token_data);

    // Описание загрузки объекта в корзину
    // https://cloud.google.com/storage/docs/json_api/v1/objects/insert
    const BUCKET_NAME: &str = "dev_test_public_bucket";
    const TEST_DATA: &[u8] = b"test test test";
    let file_name = format!("{}.txt", uuid::Uuid::new_v4());

    // Адрес запроса
    let uri = Uri::builder()
        .scheme("https")
        .authority(Authority::from_static("storage.googleapis.com"))
        .path_and_query(format!(
            "/upload/storage/v1/b/{}/o?name={}&uploadType=media&fields={}",
            urlencoding::encode(BUCKET_NAME),
            urlencoding::encode(&file_name),
            urlencoding::encode("md5Hash,mediaLink") // Только нужные поля в ответе сервера
        ))
        .build()
        .wrap_err("Uri build failed")?;
    debug!(?uri);

    // Объект запроса
    // https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html
    let request = Request::builder()
        .method(Method::POST)
        .version(hyper::Version::HTTP_2)
        .uri(uri)
        // Добавляется само если флаг выше true,
        // TODO: Что-то не так с установкой значения host, если выставить, то фейлится запрос
        // Может быть дело в регистре?
        // .header(header::HOST, "oauth2.googleapis.com")
        .header(header::USER_AGENT, "hyper")
        .header(header::CONTENT_LENGTH, TEST_DATA.len())
        .header(header::ACCEPT, mime::APPLICATION_JSON.to_string()) // TODO: Optimize
        .header(header::AUTHORIZATION, format!("Bearer {}", token_data.access_token))
        .header(header::CONTENT_TYPE, mime::OCTET_STREAM.to_string()) // TODO: Optimize
        .body(BodyStruct::from(TEST_DATA))
        .wrap_err("Request build error")?;
    info!(?request);

    // Описание данных ответа
    // https://cloud.google.com/storage/docs/json_api/v1/objects#resource

    // Объект ответа
    let response = http_client.request(request).await.wrap_err("Http response error")?;
    debug!(?response);

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
        let body_data = aggregate(response).await.wrap_err("Body data receive")?;
        let info = json_from_reader::<_, Info>(body_data.reader()).wrap_err("Response prasing err")?;
        info!("Uploading result: {:?}", info);
    } else {
        // Данные
        let body_data = to_bytes(response).await.wrap_err("Body data receive")?;
        info!(?body_data);
        return Err(eyre::eyre!("Invalid upload response status"));
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
