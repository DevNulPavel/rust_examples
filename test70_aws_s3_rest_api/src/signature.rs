use crate::date::DateInfo;
use eyre::WrapErr;
use hmac_sha256::HMAC;
use hyper::http::request::Builder as RequestBuilder;
use sha2::{Digest, Sha256};
use std::fmt::Write as FmtWrite;
use tracing::{debug, instrument};

//////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
struct SignatureHeaders {
    full_headers: String,
    signed_headers: String,
}

#[instrument(level = "error", skip(req_builder))]
fn build_signature_headers(req_builder: &RequestBuilder) -> Result<SignatureHeaders, eyre::Error> {
    if let Some(headers) = req_builder.headers_ref() {
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
        let mut full_headers = String::new();
        let mut signed_headers = String::new();
        for (key, val) in key_values_vec.into_iter() {
            writeln!(full_headers, "{}:{}", key, val)?;
            write!(signed_headers, "{};", key)?;
        }
        signed_headers.pop(); // Remove last ;

        Ok(SignatureHeaders {
            full_headers,
            signed_headers,
        })
    } else {
        Ok(SignatureHeaders::default())
    }
}

//////////////////////////////////////////////////////////////////////////////////////////

#[instrument(level = "error", skip(req_builder, region, service, date_info, body_sha256, aws_secret_access_key))]
pub fn calculate_signature(
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
    let SignatureHeaders {
        full_headers,
        signed_headers,
    } = build_signature_headers(req_builder)?;

    // Создаем строку от которой будет хэш
    let canonical_request_string = format!(
        "{method}\n{uri}\n{query}\n{headers}\n{signed_headers}\n{payload_hash}",
        method = method,
        uri = path_str,
        query = query,
        headers = full_headers,
        signed_headers = signed_headers,
        payload_hash = body_sha256
    );
    debug!("Request string:\n{}", canonical_request_string);

    // Считаем SHA256 от строки
    let req_hash = Sha256::digest(canonical_request_string.as_bytes());

    // Строка для подписи
    // https://docs.aws.amazon.com/general/latest/gr/sigv4-create-string-to-sign.html
    // https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-header-based-auth.html
    // ISO8601Format
    let sign_string = format!(
        "AWS4-HMAC-SHA256\n{timestamp}\n{date}/{region}/{service}/aws4_request\n{req_hash:x}",
        timestamp = date_info.timestamp_iso8601,
        date = date_info.date_yyyymmdd,
        region = region,
        service = service,
        req_hash = req_hash
    );
    debug!("Sign string:\n{}", sign_string);

    // Вычисляем подпись в виде хеша
    // TODO: оптимизации, цепочечный вызовы, вложенные вызовы
    let signature = {
        let date_key = HMAC::mac(
            date_info.date_yyyymmdd.as_bytes(),
            format!("AWS4{}", aws_secret_access_key).as_bytes(),
        );
        let date_region_key = HMAC::mac(region.as_bytes(), &date_key);
        let date_region_service_key = HMAC::mac(service.as_bytes(), &date_region_key);
        let signing_key = HMAC::mac("aws4_request".as_bytes(), &date_region_service_key);

        let signature = HMAC::mac(sign_string.as_bytes(), &signing_key);

        hex::encode(signature)
    };
    debug!(%signature, "Signature");

    Ok((signature, signed_headers))
}

#[instrument(level = "error", skip(aws_access_key_id, signature))]
pub fn make_autorization_header(
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
