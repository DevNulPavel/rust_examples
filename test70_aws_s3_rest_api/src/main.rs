use chrono::{Date, DateTime, Utc};
use eyre::{Context, ContextCompat, WrapErr};
use futures::{Stream, StreamExt};
// use hmac::{Hmac, Mac, NewMac};
use hmac_sha256::HMAC;
use http_body::Body as BodyTrait;
use hyper::{
    body::{aggregate, to_bytes, Body as BodyStruct, HttpBody},
    client::HttpConnector,
    header::{HeaderName, HeaderValue},
    http::{header, request::Builder as RequestBuilder, uri::Authority, Version},
    Client, Method, Request, Response, StatusCode, Uri,
};
use hyper_rustls::HttpsConnector;
use mime::Mime;
use sha2::{Digest, Sha256};
use std::{borrow::Cow, convert::TryFrom, fmt::Write as FmtWrite, process::exit, str::FromStr, time::Duration};
use tokio::time::timeout;
use tracing::{debug, error, info, instrument};
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

#[derive(Debug)]
struct DateInfo {
    date_time: DateTime<Utc>,
    date_yyyymmdd: String,
    timestamp_iso8601: String,
}
impl DateInfo {
    fn now() -> DateInfo {
        let date_time = Utc::now();
        let timestamp = date_time.format("%Y%m%dT%H%M%SZ").to_string();
        let date = date_time.date().format("%Y%m%d").to_string();
        DateInfo {
            date_time,
            date_yyyymmdd: date,
            timestamp_iso8601: timestamp,
        }
    }
}

#[instrument(level = "error", skip(req_builder, region, service, date_info, body_sha256, aws_secret_access_key))]
fn calculate_signature(
    req_builder: &RequestBuilder,
    region: &str,
    service: &str,
    date_info: &DateInfo,
    body_sha256: &str,
    aws_secret_access_key: &str,
) -> Result<(String, String), eyre::Error> {
    // Описание заголовка подписи
    // https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-header-based-auth.html

    // Метод в виде строки
    let method = req_builder
        .method_ref()
        .ok_or_else(|| eyre::eyre!("HTTP method do not provoded"))?
        .to_string();

    // Объект URI
    let uri_obj = req_builder.uri_ref().ok_or_else(|| eyre::eyre!("Uri is missing"))?;

    // Uri в виде строки без query параметров
    let path_str = uri_obj.path();

    // Отдельно query параметры
    let query = uri_obj.query().unwrap_or("");

    // Список заголовков
    // TODO: Use COW
    let (headers, signed_headers): (String, String) = if let Some(headers) = req_builder.headers_ref() {
        // Массив из имен заголовков в нижнем регистре и их значений, отсортированный по алфавиту
        let mut key_values_vec = Vec::new();
        key_values_vec.reserve(headers.len());
        for (name, val) in headers.iter() {
            let name = name.as_str().trim().to_lowercase();
            let val = val
                .to_str()
                .wrap_err_with(|| eyre::eyre!("Header {} convert to str error", name))?
                .trim();
            key_values_vec.push((name, val));
        }
        key_values_vec.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));

        // Объединяем
        let mut result_full_headers = String::new();
        let mut headers_list = String::new();
        for (key, val) in key_values_vec.into_iter() {
            write!(result_full_headers, "{}:{}\n", key, val)?;
            write!(headers_list, "{};", key)?;
        }
        headers_list.pop(); // Remove last ;

        (result_full_headers, headers_list)
    } else {
        (String::new(), String::new())
    };

    let canonical_request_string = format!(
        "{method}\n\
            {uri}\n\
            {query}\n\
            {headers}\n\
            {signed_headers}\n\
            {payload_hash}",
        method = method,
        uri = path_str,
        query = query,
        headers = headers,
        signed_headers = signed_headers,
        payload_hash = body_sha256
    );
    debug!("Request string:\n{}", canonical_request_string);

    // Строка для подписи
    // https://docs.aws.amazon.com/general/latest/gr/sigv4-create-string-to-sign.html
    // https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-header-based-auth.html
    // ISO8601Format
    let req_hash = Sha256::digest(canonical_request_string.as_bytes());
    let sign_string = format!(
        "AWS4-HMAC-SHA256\n\
        {timestamp}\n\
        {date}/{region}/{service}/aws4_request\n\
        {req_hash:x}",
        timestamp = date_info.timestamp_iso8601,
        date = date_info.date_yyyymmdd,
        region = region,
        service = service,
        req_hash = req_hash
    );
    debug!("Sign string:\n{}", sign_string);

    // Вычисляем подпись в виде хеша
    // TODO: оптимизации, цепочечный вызовы, вложенные вызовы
    let date_key = {
        HMAC::mac(date_info.date_yyyymmdd.as_bytes(), format!("AWS4{}", aws_secret_access_key).as_bytes())
        // let mut sha = Hmac::<sha2::Sha256>::new_from_slice()?;
        // sha.update();
        // sha.update(date_info.date_yyyymmdd.as_bytes());
        // let date_key = sha.finalize_reset();
    };
    let date_region_key = {
        // let mut sha = Sha256::new();
        // sha.update(date_key);
        // let mut sha = Hmac::<Sha256>::new(date_key.into_bytes());
        // sha.update(region.as_bytes());
        // sha.finalize()
        HMAC::mac(region.as_bytes(), &date_key)
    };
    let date_region_service_key = {
        // let mut sha = Sha256::new();
        // sha.update(date_region_key);
        // sha.update(service);
        // sha.finalize()
        HMAC::mac(service.as_bytes(), &date_region_key)
    };
    let signing_key = {
        // let mut sha = Sha256::new();
        // sha.update(date_region_service_key);
        // sha.update("aws4_request");
        // sha.finalize()
        HMAC::mac("aws4_request".as_bytes(), &date_region_service_key)
    };
    debug!("Sign key: {:x?}", signing_key);

    let signature = {
        // let mut sha = Sha256::new();
        // sha.update(signing_key);
        // sha.update(sign_string);
        // sha.finalize()
        HMAC::mac(sign_string.as_bytes(), &signing_key)
    };

    Ok((hex::encode(signature), signed_headers))
}

#[instrument(level = "error", skip(aws_access_key_id, signature))]
fn make_autorization_header(
    aws_access_key_id: &str,
    region: &str,
    service: &str,
    date_info: &DateInfo,
    signed_headers: &str,
    signature: &str,
) -> String {
    // Описание
    // https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-auth-using-authorization-header.html
    // https://docs.aws.amazon.com/general/latest/gr/sigv4-create-string-to-sign.html
    let result_string = format!(
        "AWS4-HMAC-SHA256 \
        Credential={access_key}/{date}/{region}/{service}/aws4_request,\
        SignedHeaders={signed_headers},\
        Signature={signature}",
        access_key = aws_access_key_id,
        date = date_info.date_yyyymmdd,
        region = region,
        service = service,
        signed_headers = signed_headers,
        signature = signature
    );
    result_string
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
    let file_name = "target_file_name.txt";
    let test_data = "test test test";

    let host = format!("{}.s3.amazonaws.com", bucket);
    let path_query = format!("/{}", file_name);
    let test_data_sha = format!("{:x}", Sha256::digest(test_data.as_bytes()));

    // Адрес запроса
    let uri = Uri::builder()
        .scheme("https")
        .authority(Authority::from_str(&host).wrap_err("Authority parse error")?)
        .path_and_query(path_query) // TODO: URL-encode
        .build()
        .wrap_err("Uri build failed")?;
    info!(?uri, "Uri");

    // Объект запроса
    // https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutObject.html
    let request = Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::HOST, host)
        // .header(header::DATE, date_info.date_time.to_rfc2822())
        .header(header::CONTENT_LENGTH, test_data.len())
        .header(header::CONTENT_TYPE, mime::TEXT_PLAIN.to_string()) // TODO: Optimize
        .header("x-amz-date", &date_info.timestamp_iso8601)
        .header("x-amz-content-sha256", &test_data_sha);

    // Authorization header с подписью
    let (signature, signed_headers) = calculate_signature(&request, region, service, &date_info, &test_data_sha, &aws_secret_access_key)
        .wrap_err("Signature calculate error")?;
    let autorization_header = make_autorization_header(&aws_access_key_id, region, service, &date_info, &signed_headers, &signature);

    let request = request
        .header("Authorization", autorization_header)
        .body(BodyStruct::from(test_data))
        .wrap_err("Request build error")?;
    info!(?request, "Request");

    // Объект ответа
    let response = http_client.request(request).await.wrap_err("Http response error")?;
    info!(?response, "Response");

    // Получаем длину контента
    // let content_length: usize = response
    //     .headers()
    //     .get(header::CONTENT_LENGTH)
    //     .ok_or_else(|| eyre::eyre!("Content length header is missing"))?
    //     .to_str()?
    //     .parse()
    //     .wrap_err("Content Length parse failed")?;
    // info!(content_length, "Content length");

    // Получаем тип контента
    // let content_type_mime: Mime = response
    //     .headers()
    //     .get(header::CONTENT_TYPE)
    //     .ok_or_else(|| eyre::eyre!("Content type header is missing"))?
    //     .to_str()
    //     .wrap_err("Content type header to string convert failed")?
    //     .parse()
    //     .wrap_err("Mime parse failed")?;
    // info!(?content_type_mime, "Content type");

    // Работаем с ответом
    // if content_type_mime == mime::APPLICATION_JSON {
    // } else if content_type_mime == mime::TEXT_PLAIN {
    // } else {
    //     eyre::eyre!("Unknown content type");
    // }

    let body = response.into_body();

    // match timeout(Duration::from_secs(1), body.trailers()).await {
    //     Ok(trailers) => {
    //         let trailers = trailers.wrap_err("Trailers receive error")?;
    //         info!(?trailers, "Body trailers");
    //     }
    //     Err(_) => {
    //         info!("Body trailers timeout");
    //     }
    // }

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
