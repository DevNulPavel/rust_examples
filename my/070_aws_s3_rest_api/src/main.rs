mod date;
mod signature;

use eyre::WrapErr;
// use futures::{Stream, StreamExt};
// use http_body::Body as BodyTrait;
use hyper::{
    body::{to_bytes, Body as BodyStruct},
    http::{header, uri::Authority},
    Client, Method, Request, Response, Uri,
};
use hyper_rustls::HttpsConnector;
use mime::Mime;
use crate::{
    date::DateInfo,
    signature::{calculate_signature, make_autorization_header},
};
use sha2::{Digest, Sha256};
use std::{process::exit, str::FromStr};
use tracing::{error, info};
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

fn get_content_length(response: &Response<BodyStruct>) -> Result<Option<usize>, eyre::Error> {
    let content_length: Option<usize> = match response.headers().get(header::CONTENT_LENGTH) {
        Some(val) => {
            let num = val
                .to_str()
                .wrap_err("Content-Length string convert failed")?
                .parse::<usize>()
                .wrap_err("Content Length parse failed")?;
            Some(num)
        }
        None => None,
    };
    Ok(content_length)
}

fn get_content_type(response: &Response<BodyStruct>) -> Result<Option<Mime>, eyre::Error> {
    let header_val = match response.headers().get(header::CONTENT_TYPE){
        Some(val) => val,
        None => return Ok(None)
    };
    let content_type_mime: Mime = header_val
        .to_str()
        .wrap_err("Content type header to string convert failed")?
        .parse()
        .wrap_err("Mime parse failed")?;
    Ok(Some(content_type_mime))
}

async fn execute_app() -> Result<(), eyre::Error> {
    // Переменные окружения для Amazon
    let aws_access_key_id = std::env::var("AWS_ACCESS_KEY_ID").wrap_err("AWS_ACCESS_KEY_ID value is missing")?;
    let aws_secret_access_key = std::env::var("AWS_SECRET_ACCESS_KEY").wrap_err("AWS_SECRET_ACCESS_KEY value is missing")?;

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

    // TODO: Заголовки
    // x-amz-content-sha256 - UNSIGNED-PAYLOAD, https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-auth-using-authorization-header.html
    // Authorization - https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-auth-using-authorization-header.html
    // Date

    // Переменные
    let bucket = "public-files-bucket";
    let date_info = DateInfo::now();
    let region = "eu-west-2";
    let service = "s3";
    let file_name = format!("{}.txt", uuid::Uuid::new_v4());
    let test_data = "test test test";

    let file_url = format!("{}.s3.{}.amazonaws.com", bucket, region);
    let path_query = format!("/{}", file_name);
    let test_data_sha = format!("{:x}", Sha256::digest(test_data.as_bytes()));

    // Адрес запроса
    let uri = Uri::builder()
        .scheme("https")
        .authority(Authority::from_str(&file_url).wrap_err("Authority parse error")?)
        .path_and_query(path_query) // TODO: URL-encode
        .build()
        .wrap_err("Uri build failed")?;
    info!(?uri, "Uri");

    // Объект запроса
    // https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html
    let request = Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::HOST, file_url)
        .header(header::DATE, &date_info.timestamp_rfc2822)
        .header(header::CONTENT_LENGTH, test_data.len())
        .header(header::CONTENT_TYPE, mime::TEXT_PLAIN.to_string()) // TODO: Optimize
        .header("x-amz-date", &date_info.timestamp_iso8601)
        .header("x-amz-acl", "public-read")
        // .header("x-amz-content-sha256", "UNSIGNED-PAYLOAD");
        .header("x-amz-content-sha256", &test_data_sha);

    // Authorization header с подписью
    let autorization_header = {
        let (signature, signed_headers) =
            calculate_signature(&request, region, service, &date_info, &test_data_sha, &aws_secret_access_key)
                .wrap_err("Signature calculate error")?;
        make_autorization_header(&aws_access_key_id, region, service, &date_info, &signed_headers, &signature)
    };

    // Добавляем подпись и тело
    let request = request
        .header("Authorization", autorization_header)
        .body(BodyStruct::from(test_data))
        .wrap_err("Request build error")?;
    info!(?request, "Request");

    // Объект ответа
    let response = http_client.request(request).await.wrap_err("Http response error")?;
    info!(?response, "Response");

    // Статус HTTP
    let status = response.status();

    // Получаем длину контента
    let content_length: Option<usize> = get_content_length(&response).wrap_err("Content type receive err")?;
    info!(?content_length, "Content length");

    // Получаем тип контента
    let content_type_mime: Option<Mime> = get_content_type(&response).wrap_err("Content type receive err")?;
    info!(?content_type_mime, "Content type");

    // В зависимости от статуса обрабатыаем иначе
    if status.is_success() {
        let result_file_url = format!("https://{}.s3.{}.amazonaws.com/{}", bucket, region, file_name);
        info!(%result_file_url, "Result file url");
    }else if status.is_redirection() {
        // TODO: Нужно повторно выполнить запрос, но актуально только для получения вроде бы
    }else{
        // Работаем с ответом
        // if content_type_mime == mime::APPLICATION_JSON {
        // } else if content_type_mime == mime::TEXT_PLAIN {
        // } else {
        //     eyre::eyre!("Unknown content type");
        // }

        let body = response.into_body();

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
