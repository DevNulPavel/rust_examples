use eyre::WrapErr;
use futures::{future::pending, TryStreamExt};
use hyper::{
    body::to_bytes,
    body::Bytes,
    header,
    server::conn::{AddrStream, Http},
    service::{make_service_fn, service_fn},
    Body as BodyStruct, Method, Request, Response, Server, StatusCode,
};
use std::{borrow::Cow, convert::Infallible, error::Error as StdError, fmt::Display, net::SocketAddr, process::exit};
use tracing::{debug, error, info, instrument};
use tracing_error::ErrorLayer;
use tracing_log::{log::warn, LogTracer};
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

macro_rules! unwrap_or_fail_resp {
    ($code: expr) => {
        match $code {
            Ok(val) => val,
            Err(err) => {
                // Выводим ошибку
                error!("{}", err);

                // Создаем ответ с правильным статусом
                let resp = response_with_status_and_empty_body(StatusCode::INTERNAL_SERVER_ERROR);

                // Выходим с ошибкой
                return Ok(resp);
            }
        }
    };
    ($code: expr, $err_status: expr) => {
        match $code {
            Ok(val) => val,
            Err(err) => {
                // Выводим ошибку
                error!("{}", err);

                // Создаем ответ с правильным статусом
                let resp = response_with_status_and_empty_body($err_status);

                // Выходим с ошибкой
                return Ok(resp);
            }
        }
    };
}

macro_rules! true_or_fail_resp {
    ($code: expr, $err_status: expr, $desc: literal) => {
        if !$code {
            error!($desc);
            return Ok(response_with_status_end_error($err_status, $desc));
        }
    };
}

fn response_with_status_and_empty_body(status: StatusCode) -> Response<BodyStruct> {
    Response::builder()
        .status(status)
        .body(BodyStruct::empty())
        .expect("Static fail response create failed") // Статически создаем ответ, здесь не критично
}

fn response_with_status_end_error(status: StatusCode, err_desc: &str) -> Response<BodyStruct> {
    let error_json = format!(r#"{{"description": "{}"}}"#, err_desc);
    Response::builder()
        .status(status)
        .body(BodyStruct::from(error_json))
        .expect("Static fail response create failed") // Статически создаем ответ, здесь не критично
}

#[derive(Debug)]
struct ErrorWithStatusAndDesc {
    // Время жизни распространяется лишь на ссылки в подтипе, они должны иметь время жизни 'static
    // На обычные переменные не распространяется
    source: Option<Box<dyn StdError + Send + Sync + 'static>>,
    status: StatusCode,
    desc: Cow<'static, str>,
}
impl ErrorWithStatusAndDesc {
    fn from_error<E: StdError + Send + Sync + 'static>(e: E, status: StatusCode, desc: &'static str) -> Self {
        ErrorWithStatusAndDesc {
            source: Some(Box::new(e)),
            status,
            desc: Cow::Borrowed(desc),
        }
    }
}
impl Display for ErrorWithStatusAndDesc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Status: {}, Description: {}", self.status, self.desc)
    }
}
impl StdError for ErrorWithStatusAndDesc {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as &(dyn StdError + 'static))
    }
}

#[instrument(level = "error")]
async fn service_handler(remove_addr: SocketAddr, req: Request<BodyStruct>) -> Result<Response<BodyStruct>, eyre::Error> {
    // debug!("Request processing begin");

    match (req.method(), req.uri().path()) {
        // Запрос к корню
        (&Method::GET, "/") => {
            info!("Root");
            let response = Response::builder()
                .status(StatusCode::MOVED_PERMANENTLY)
                .header(header::LOCATION, "/help")
                .body(BodyStruct::empty())?;

            Ok(response)
        }

        // Помощь
        (&Method::GET, "/help") => {
            info!("Help");
            let response = Response::builder()
                .status(StatusCode::OK)
                .body(BodyStruct::from("Try to send POST request at '/echo'"))?;
            Ok(response)
        }

        // Возвращаем все в ответ
        (&Method::POST, "/echo") => {
            info!("Echo");
            let resp = Response::builder().status(StatusCode::OK).body(req.into_body())?;
            Ok(resp)
        }

        // Возвращаем все в ответ в верхнем регистре
        (&Method::POST, "/echo/uppercase") => {
            info!("Echo uppercase");

            let new_stream = req.into_body().map_ok(|data_chunk| {
                let result: Bytes = data_chunk.into_iter().map(|val| val.to_ascii_uppercase()).collect();
                result
            });

            let resp = Response::builder()
                .status(StatusCode::OK)
                .body(BodyStruct::wrap_stream(new_stream))
                .wrap_err("Body build")?;
            Ok(resp)
        }

        // Возвращаем все в ответ в обратном порядке
        (&Method::POST, "/echo/reverse") => {
            info!("Echo reverse");

            let body_data = to_bytes(req.into_body())
                .await
                .map_err(|e| ErrorWithStatusAndDesc::from_error(e, StatusCode::BAD_GATEWAY, "Body to bytes convert failed"))
                .wrap_err("Failed to convert body stream to bytes")?;

            let rev_body_data: Bytes = body_data.iter().rev().cloned().collect();

            let resp = Response::builder()
                .status(StatusCode::OK)
                .body(BodyStruct::from(rev_body_data))
                .wrap_err("Body build")?;
            Ok(resp)
        }

        // Возвращаем все в ответ в обратном порядке
        (&Method::POST, "/web_socket") => {
            info!("Web socket");

            // Проверим, что в заголовках есть заголовок UPGRADE, либо выходим с ошибкой
            фыв
            true_or_fail_resp!(
                req.headers().contains_key(header::UPGRADE),
                StatusCode::UPGRADE_REQUIRED,
                "UPGRADE header is missing"
            );

            let resp = unwrap_or_fail_resp!(Response::builder().status(StatusCode::OK).body(BodyStruct::empty()));
            Ok(resp)
        }

        // Любой другой запрос
        _ => {
            warn!("Invalid request");
            let response = Response::builder().status(StatusCode::NOT_FOUND).body(BodyStruct::empty()).unwrap();
            Ok(response)
        }
    }
}

async fn shutdown_signal_wait() {
    // Wait for the CTRL+C signal
    if let Err(err) = tokio::signal::ctrl_c().await {
        warn!("Shutdown signal awaiter setup failed, continue without: {}", err);
        // Создаем поэтому вечную future
        pending::<()>().await;
    }
    info!("Shutdown signal received");
}

async fn execute_app() -> Result<(), eyre::Error> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    // Сервис необходим для каждого соединения, поэтому создаем враппер, который будет генерировать наш сервис
    let make_svc = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr();
        async move {
            // Создаем сервис из функции с помощью service_fn
            Ok::<_, Infallible>(service_fn(move |req| async move {
                match service_handler(remote_addr, req).await {
                    resp @ Ok(_) => resp,
                    Err(err) => {
                        // Выводим ошибку в консоль
                        error!("{}", err);

                        // Ошибка с дополнительной инфой?
                        let resp = if let Some(http_err) = err.downcast_ref::<ErrorWithStatusAndDesc>() {
                            // Создаем ответ с правильным статусом
                            response_with_status_end_error(http_err.status, http_err.desc.as_ref())
                        } else {
                            // Создаем ответ со стандартным статусом
                            response_with_status_and_empty_body(StatusCode::INTERNAL_SERVER_ERROR)
                        };
                        Ok(resp)
                    }
                }
            }))
        }
    });

    // Создаем сервер
    Server::bind(&addr)
        .serve(make_svc)
        .with_graceful_shutdown(shutdown_signal_wait())
        .await
        .wrap_err("Server awaiting")?;

    Ok(())
}

#[tokio::main]
async fn main() {
    // Бектрейсы в ошибках
    color_eyre::install().unwrap();

    // Логи
    initialize_logs().unwrap();

    if let Err(err) = execute_app().await {
        error!("{:?}", err);
        exit(1);
    } else {
        exit(0);
    }
}
