use crate::{
    error::{ErrorWithStatusAndDesc, WrapErrorWithStatusAndDesc},
    types::App,
};
use futures::StreamExt;
use hyper::{
    body::{aggregate, to_bytes, Body as BodyStruct, Buf},
    http::{
        header,
        method::Method,
        status::StatusCode,
        uri::{Authority, Uri},
    },
    Request, Response,
};
// use pin_project::pin_project;
use serde::Deserialize;
use serde_json::from_reader as json_from_reader;
use tracing::{debug, error, info, instrument};

/////////////////////////////////////////////////////////////////////////////////////////////////////////

/// A wrapper around any type that implements [`Stream`](futures::Stream) to be
/// compatible with async_compression's Stream based encoders
/*#[pin_project]
#[derive(Debug)]
pub struct CompressableBody<S, E>
where
    E: std::error::Error,
    S: Stream<Item = Result<Bytes, E>>,
{
    #[pin]
    body: S,
}

impl<S, E> Stream for CompressableBody<S, E>
where
    E: std::error::Error,
    S: Stream<Item = Result<Bytes, E>>,
{
    type Item = std::io::Result<Bytes>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use std::io::{Error, ErrorKind};

        let pin = self.project();
        S::poll_next(pin.body, cx).map_err(|_| Error::from(ErrorKind::InvalidData))
    }
}
impl From<BodyStruct> for CompressableBody<BodyStruct, hyper::Error> {
    fn from(body: BodyStruct) -> Self {
        CompressableBody { body }
    }
}*/

/////////////////////////////////////////////////////////////////////////////////////////////////////////

fn build_upload_uri(bucket_name: &str, file_name: &str) -> Result<Uri, hyper::http::Error> {
    Uri::builder()
        .scheme("https")
        .authority(Authority::from_static("storage.googleapis.com"))
        .path_and_query(format!(
            "/upload/storage/v1/b/{}/o?name={}&uploadType=media&fields={}",
            urlencoding::encode(bucket_name),
            urlencoding::encode(file_name),
            urlencoding::encode("id,md5Hash,mediaLink") // Только нужные поля в ответе сервера, https://cloud.google.com/storage/docs/json_api/v1/objects#resource
        ))
        .build()
}

fn build_upload_request(uri: Uri, token: String, body: BodyStruct) -> Result<Request<BodyStruct>, hyper::http::Error> {
    Request::builder()
        .method(Method::POST)
        .version(hyper::Version::HTTP_2)
        .uri(uri)
        // TODO: Что-то не так с установкой значения host, если выставить, то фейлится запрос
        // Может быть дело в регистре?
        // .header(header::HOST, "oauth2.googleapis.com")
        .header(header::USER_AGENT, "hyper")
        // .header(header::CONTENT_LENGTH, data_length)
        .header(header::ACCEPT, mime::APPLICATION_JSON.to_string()) // TODO: Optimize
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, mime::OCTET_STREAM.to_string()) // TODO: Optimize
        .body(body)
}

#[derive(Debug, Deserialize)]
struct UploadResultData {
    id: String,
    #[serde(rename = "md5Hash")]
    md5: String,
    #[serde(rename = "mediaLink")]
    link: String,
}

async fn parse_response_body(response: Response<BodyStruct>) -> Result<UploadResultData, ErrorWithStatusAndDesc> {
    let body_data = aggregate(response)
        .await
        .wrap_err_with_status_desc(StatusCode::INTERNAL_SERVER_ERROR, "Google cloud response receive failed".into())?;

    let info = json_from_reader::<_, UploadResultData>(body_data.reader())
        .wrap_err_with_status_desc(StatusCode::INTERNAL_SERVER_ERROR, "Google cloud response parsing failed".into())?;

    Ok(info)
}

#[instrument(level = "error", skip(app, req))]
async fn file_upload(app: &App, req: Request<BodyStruct>) -> Result<Response<BodyStruct>, ErrorWithStatusAndDesc> {
    info!("File uploading");

    // Получаем токен из запроса и проверяем
    let token = req
        .headers()
        .get("X-Api-Token")
        .wrap_err_with_status_desc(StatusCode::UNAUTHORIZED, "Api token is missing".into())
        .and_then(|val| {
            std::str::from_utf8(val.as_bytes()).wrap_err_with_status_desc(StatusCode::UNAUTHORIZED, "Api token parsing failed".into())
        })?;
    if token != app.app_arguments.uploader_api_token {
        return Err(ErrorWithStatusAndDesc::new_with_status_desc(
            StatusCode::UNAUTHORIZED,
            "Invalid api token".into(),
        ));
    }

    // Получаем размер данных исходных
    /*let data_length = get_content_length(req.headers())
    .wrap_err_with_status_desc(StatusCode::LENGTH_REQUIRED, "Content-Length header parsing failed".into())?
    .wrap_err_with_status_desc(StatusCode::LENGTH_REQUIRED, "Content-Length header is missing".into())?;*/

    // Получаем токен для Google API
    let token = app
        .token_provider
        .get_token()
        .await
        .wrap_err_with_status_desc(StatusCode::UNAUTHORIZED, "Google cloud token receive failed".into())?;

    // TODO: Время жизни для файликов на сервере

    // Адрес запроса
    let uri = {
        // Имя нашего файлика
        let file_name = format!("{:x}.txt.gz", uuid::Uuid::new_v4());
        // Адрес
        build_upload_uri(&app.app_arguments.google_bucket_name, &file_name).wrap_err_with_500()?
    };
    debug!("Request uri: {}", uri);

    // Здесь же можно сделать шифрование данных перед компрессией
    let body_stream = req
        .into_body()
        .map(|v| v.map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput)));
    let reader = tokio_util::io::StreamReader::new(body_stream);
    let compressor = async_compression::tokio::bufread::GzipEncoder::new(reader);
    let out_stream = tokio_util::io::ReaderStream::new(compressor);

    // Объект запроса
    let request = build_upload_request(uri, token, BodyStruct::wrap_stream(out_stream)).wrap_err_with_500()?;
    debug!("Request object: {:?}", request);

    // Объект ответа
    let response = app
        .http_client
        .request(request)
        .await
        .wrap_err_with_status_desc(StatusCode::INTERNAL_SERVER_ERROR, "Google cloud error".into())?;
    debug!("Google response: {:?}", response);

    // Статус
    let status = response.status();
    debug!("Response status: {:?}", status);

    // Обрабатываем в зависимости от ответа
    if status.is_success() {
        // Данные парсим
        let info = parse_response_body(response).await?;
        debug!("Uploading result: {:?}", info);

        // Формируем ответ
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
        let body_data = to_bytes(response)
            .await
            .wrap_err_with_status_desc(StatusCode::INTERNAL_SERVER_ERROR, "Google cloud response receive failed".into())?;
        error!("Upload fail result: {:?}", body_data);

        // Если есть внятный ответ - пробрасываем его
        match std::str::from_utf8(&body_data).ok() {
            Some(text) => {
                error!("Upload fail result text: {}", text);
                let resp = format!("Google error response: {}", text);
                return Err(ErrorWithStatusAndDesc::new_with_status_desc(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    resp.into(),
                ));
            }
            None => {
                return Err(ErrorWithStatusAndDesc::new_with_status_desc(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Google uploading failed".into(),
                ));
            }
        }
    }
}

// Трассировка настраивается уровнем выше
// #[instrument(level = "error")]
pub async fn handle_request(app: &App, req: Request<BodyStruct>) -> Result<Response<BodyStruct>, ErrorWithStatusAndDesc> {
    // debug!("Request processing begin");

    match (req.method(), req.uri().path()) {
        // Отладочным образом получаем токен
        /*(&Method::GET, "/token") => {
            info!("Token");

            let token = app
                .token_provider
                .get_token()
                .await
                .wrap_err_with_status_desc(StatusCode::UNAUTHORIZED, "Google cloud token receive failed".into())?;

            let json_text = format!(r#"{{"token": "{}"}}"#, token);

            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.essence_str()) // TODO: Check
                // .header(header::CONTENT_LENGTH, json_text.as_bytes().len())
                .body(BodyStruct::from(json_text))
                .wrap_err_with_500()?;
            Ok(response)
        }*/
        // Выгружаем данные в Cloud
        (&Method::POST, "/upload_file") => file_upload(app, req).await,

        // Любой другой запрос
        _ => {
            error!("Invalid request");
            return Err(ErrorWithStatusAndDesc::new_with_status_desc(
                StatusCode::BAD_REQUEST,
                "Wrong path".into(),
            ));
        }
    }
}
