mod service_account;

use eyre::WrapErr;
use http_body::Body;
use rsa::pkcs1::FromRsaPrivateKey;
// use futures::{Stream, StreamExt};
// use http_body::Body as BodyTrait;
use crate::service_account::ServiceAccountData;
use chrono::{DateTime, Duration, Utc};
use hyper::{
    body::{to_bytes, Body as BodyStruct},
    http::{header, uri::Authority},
    Client, Method, Request, Response, Uri,
};
use hyper_rustls::HttpsConnector;
use mime::Mime;
use rsa::{pkcs8::FromPrivateKey, PaddingScheme, RsaPrivateKey};
use std::{convert::TryFrom, path::Path, process::exit, str::FromStr};
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
    let header_val = match response.headers().get(header::CONTENT_TYPE) {
        Some(val) => val,
        None => return Ok(None),
    };
    let content_type_mime: Mime = header_val
        .to_str()
        .wrap_err("Content type header to string convert failed")?
        .parse()
        .wrap_err("Mime parse failed")?;
    Ok(Some(content_type_mime))
}

async fn execute_app() -> Result<(), eyre::Error> {
    // Описание логики получения токена из сервисного аккаунта
    // https://developers.google.com/identity/protocols/oauth2/service-account#httprest
    // https://cloud.google.com/storage/docs/authentication

    const SCOPES: &str = "https://www.googleapis.com/auth/devstorage.read_write";

    // Данные по сервисному аккаунту
    let service_acc_data =
        ServiceAccountData::new_from_file(Path::new("./env/test_google_service_account.json")).wrap_err("Service account read")?;
    debug!("Service account data: {:?}", service_acc_data);

    // TODO: Все обязательно кодируем в base64
    let jwt_result = {
        // Header
        let jwt_header = r#"{"alg":"RS256","typ":"JWT"}"#; // TODO: Строка константная, закешировать
        debug!(%jwt_header);
        let jwt_header = base64::encode(jwt_header);

        // Claims
        let current_time = Utc::now();
        let expire_time = current_time
            .checked_add_signed(Duration::minutes(59))
            .ok_or_else(|| eyre::eyre!("Expire time calc err"))?;
        let jwt_claims = format!(
            r###"{{"iss":"{}","scope":"{}","aud":"{}","exp":{},"iat":{}}}"###,
            service_acc_data.client_email,
            SCOPES,
            service_acc_data.token_uri,
            expire_time.timestamp(),
            current_time.timestamp()
        );
        debug!(%jwt_claims);
        let jwt_claims = base64::encode(jwt_claims);

        // Исходная строка для подписи
        let jwt_string_for_signature = format!("{}.{}", jwt_header, jwt_claims);
        debug!(%jwt_string_for_signature);

        // Приватный ключ читаем
        // Sign the UTF-8 representation of the input using SHA256withRSA (also known as RSASSA-PKCS1-V1_5-SIGN with the SHA-256 hash function) with the private key obtained from the Google API Console.
        // Вроде бы как метод шифрования записан в самом ключе, поэтому используем pkcs8 способ чтения закрытого ключа
        let private_key = RsaPrivateKey::from_pkcs8_pem(&service_acc_data.private_key).wrap_err("Private key parsing failed")?;
        private_key.validate().wrap_err("Private key is invalid")?;

        // Вычисляем подпись
        // use rsa::PublicKey;
        // let mut rng = rand::rngs::OsRng;
        // let public_key = private_key.to_public_key();
        // let padding = PaddingScheme::new_oaep::<sha2::Sha256>();
        // let padding = PaddingScheme::new_pkcs1v15_sign(None);
        // let signature = public_key.encrypt(&mut rng, padding, jwt_string_for_signature.as_bytes()).wrap_err("Signature encrypt failed")?;
        // let padding = PaddingScheme::new_oaep::<sha2::Sha256>();
        let padding = PaddingScheme::new_pkcs1v15_sign(None);
        use sha2::Digest;
        let signature = private_key
            .sign(padding, sha2::Sha256::digest(jwt_string_for_signature.as_bytes()).as_slice())
            .wrap_err("Sign failed")?;

        // let rand = ring::rand::SystemRandom::new();
        // let key = ring::signature::RsaKeyPair::from_pkcs8(service_acc_data.private_key.as_bytes())
        //     .map_err(|e| eyre::eyre!("Private key read failed: {}", e))?;
        // let mut signature = vec![0; key.public_modulus_len()];
        // key.sign(
        //     &ring::signature::RSA_PKCS1_SHA256,
        //     &rand,
        //     jwt_string_for_signature.as_bytes(),
        //     &mut signature,
        // )
        // .map_err(|e| eyre::eyre!("Sign failed: {}", e))?;

        // Base64 подписи
        let base_64_signature = base64::encode(signature);

        // Результат
        format!("{}.{}", jwt_string_for_signature, base_64_signature)
    };
    info!("JWT result: {}", jwt_result);

    // Коннектор для работы уже с HTTPS
    let https_connector = HttpsConnector::with_native_roots();

    // Клиент с коннектором
    let http_client = Client::builder()
        .set_host(false)
        .build::<_, BodyStruct>(https_connector);

    // Адрес запроса
    let uri = Uri::builder()
        .scheme("https")
        .authority(Authority::from_str("oauth2.googleapis.com").wrap_err("Authority parse error")?)
        .path_and_query("/token") // TODO: URL-encode
        .build()
        .wrap_err("Uri build failed")?;
    debug!("Uri: {:?}", uri);

    // Form data - это аналог query строки, но в body
    // Значения разделяются с помощью &, каждый параметр должен быть urlencoded
    let body_data = {
        let grand_type = urlencoding::encode("urn:ietf:params:oauth:grant-type:jwt-bearer"); // TODO: Optimize
        format!("grant_type={}&assertion={}", grand_type, jwt_result)
    };
    debug!("Request body: {}", body_data);

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
        .header(header::CONTENT_LENGTH, body_data.len())
        .header(header::ACCEPT, mime::APPLICATION_JSON.to_string()) // TODO: Optimize
        .header(header::USER_AGENT, "hyper")
        .header(header::CONTENT_TYPE, mime::APPLICATION_WWW_FORM_URLENCODED.to_string()) // TODO: Optimize
        .body(BodyStruct::from(body_data))
        .wrap_err("Request build error")?;
    info!("Request: {:?}", request);

    // Объект ответа
    let response = http_client.request(request).await.wrap_err("Http response error")?;
    info!("Response: {:?}", response);

    // Статус HTTP
    let status = response.status();

    // Получаем длину контента
    let content_length: Option<usize> = get_content_length(&response).wrap_err("Content type receive err")?;
    info!(?content_length, "Content length");

    // Получаем тип контента
    let content_type_mime: Option<Mime> = get_content_type(&response).wrap_err("Content type receive err")?;
    info!(?content_type_mime, "Content type");

    let body = response.into_body();
    let body_data = to_bytes(body).await.wrap_err("Body data receive")?;
    info!("Body data: {:?}", body_data);

    /*// В зависимости от статуса обрабатыаем иначе
    if status.is_success() {
    }else if status.is_redirection() {
        // TODO: Нужно повторно выполнить запрос, но актуально только для получения вроде бы
    }else{
        // Работаем с ответом
        // if content_type_mime == mime::APPLICATION_JSON {
        // } else if content_type_mime == mime::TEXT_PLAIN {
        // } else {
        //     eyre::eyre!("Unknown content type");
        // }

        // let body_data = aggregate(body).await.wrap_err("Body data receive")?;
        // info!(?body_data, "Body data");

        // while let Some(body_data) = body.data().await{
        //     let body_data = body_data?;
        //     info!(?body_data, "Body data");
        // }

        // while let Some(data) = body.next().await {
        //     info!(?data, "Body chunk");
        // }
    }*/

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
