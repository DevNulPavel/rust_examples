use crate::{
    error::{ErrorWithStatusAndDesc, WrapErrorWithStatusAndDesc},
    helpers::get_content_length,
    types::App,
};
use futures::Stream;
use hyper::{
    body::{aggregate, to_bytes, Body as BodyStruct, Buf, Bytes},
    http::{
        header,
        method::Method,
        status::StatusCode,
        uri::{Authority, Uri},
    },
    Request, Response,
};
use pin_project::pin_project;
use serde::Deserialize;
use serde_json::from_reader as json_from_reader;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tracing::{debug, error, info, instrument, trace, warn};

/////////////////////////////////////////////////////////////////////////////////////////////////////////

/// A wrapper around any type that implements [`Stream`](futures::Stream) to be
/// compatible with async_compression's Stream based encoders
#[pin_project]
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
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////

#[instrument(level = "error", skip(app, req))]
async fn file_upload(app: &App, req: Request<BodyStruct>) -> Result<Response<BodyStruct>, ErrorWithStatusAndDesc> {
    info!("File uploading");

    // TODO: Получаем токен из запроса и проверяем

    // Получаем размер данных
    /*let data_length = get_content_length(req.headers())
    .wrap_err_with_status_desc(StatusCode::LENGTH_REQUIRED, "Content-Length header parsing failed".into())?
    .wrap_err_with_status_desc(StatusCode::LENGTH_REQUIRED, "Content-Length header is missing".into())?;*/

    // Получаем токен для Google API
    let token = app
        .token_provider
        .get_token()
        .await
        .wrap_err_with_status_desc(StatusCode::UNAUTHORIZED, "Google cloud token receive failed".into())?;

    // Имя нашего файлика
    let file_name = format!("{:x}.txt.gz", uuid::Uuid::new_v4());

    // TODO: Время жизни для файликов на сервере

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

    let reader = tokio_util::io::StreamReader::new(CompressableBody::from(req.into_body()));
    let compressor = async_compression::tokio::bufread::GzipEncoder::new(reader);
    let out_stream = tokio_util::io::ReaderStream::new(compressor);

    // Объект запроса
    let request = Request::builder()
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
        .body(BodyStruct::wrap_stream(out_stream))
        .wrap_err_with_500()?;
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
        #[derive(Debug, Deserialize)]
        struct Info {
            #[serde(rename = "md5Hash")]
            md5: String,
            #[serde(rename = "mediaLink")]
            link: String,
        }
        // Данные
        let body_data = aggregate(response)
            .await
            .wrap_err_with_status_desc(StatusCode::INTERNAL_SERVER_ERROR, "Google cloud response receive failed".into())?;
        let info = json_from_reader::<_, Info>(body_data.reader())
            .wrap_err_with_status_desc(StatusCode::INTERNAL_SERVER_ERROR, "Google cloud response parsing failed".into())?;
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
        let body_data = to_bytes(response)
            .await
            .wrap_err_with_status_desc(StatusCode::INTERNAL_SERVER_ERROR, "Google cloud response receive failed".into())?;
        error!("Upload fail result: {:?}", body_data);

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
