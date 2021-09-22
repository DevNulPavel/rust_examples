mod error;
mod macroses;

use crate::error::{ErrorWithStatusAndDesc, WrapErrorWithStatusAndDesc};
use eyre::WrapErr;
use futures::{future::pending, TryStreamExt};
use hyper::{
    body::to_bytes,
    body::Bytes,
    header,
    server::conn::{AddrStream, Http},
    service::{make_service_fn, service_fn},
    upgrade::Upgraded,
    Body as BodyStruct, Method, Request, Response, Server, StatusCode,
};
use std::{borrow::Cow, convert::Infallible, error::Error as StdError, fmt::Display, net::SocketAddr, process::exit};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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

/// Handle server-side I/O after HTTP upgraded.
async fn server_upgraded_io(mut upgraded: Upgraded) -> Result<(), eyre::Error> {
    // we have an upgraded connection that we can read and
    // write on directly.
    //
    // since we completely control this example, we know exactly
    // how many bytes the client will write, so just read exact...
    let mut vec = vec![0; 7];
    upgraded.read_exact(&mut vec).await?;
    println!("server[foobar] recv: {:?}", std::str::from_utf8(&vec));

    // and now write back the server 'foobar' protocol's
    // response...
    upgraded.write_all(b"barr=foo").await?;
    println!("server[foobar] sent");
    Ok(())
}

#[instrument(level = "error")]
async fn process_web_socket(remove_addr: SocketAddr, mut req: Request<BodyStruct>) -> Result<Response<BodyStruct>, eyre::Error> {
    info!("Web socket");

    // Проверим, что в заголовках есть заголовок UPGRADE, либо выходим с ошибкой
    true_or_fail_resp!(
        req.headers().contains_key(header::UPGRADE),
        StatusCode::UPGRADE_REQUIRED,
        "UPGRADE header is missing"
    );

    // Setup a future that will eventually receive the upgraded
    // connection and talk a new protocol, and spawn the future
    // into the runtime.
    //
    // Note: This can't possibly be fulfilled until the 101 response
    // is returned below, so it's better to spawn this future instead
    // waiting for it to complete to then return a response.
    tokio::task::spawn(async move {
        match hyper::upgrade::on(&mut req).await {
            Ok(upgraded) => {
                if let Err(e) = server_upgraded_io(upgraded).await {
                    eprintln!("server foobar io error: {}", e)
                };
            }
            Err(e) => eprintln!("upgrade error: {}", e),
        }
    });

    let resp = Response::builder()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .header(header::UPGRADE, "ws") // TODO: Какое значение указывать в заголовке?
        .body(BodyStruct::empty())?;
    Ok(resp)
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
                .wrap_err_with_status(StatusCode::BAD_GATEWAY, "Body to bytes convert failed")?;

            let rev_body_data: Bytes = body_data.iter().rev().cloned().collect();

            let resp = Response::builder()
                .status(StatusCode::OK)
                .body(BodyStruct::from(rev_body_data))
                .wrap_err("Body build")?;
            Ok(resp)
        }

        // Возвращаем все в ответ в обратном порядке
        (&Method::POST, "/web_socket") => process_web_socket(remove_addr, req).await,

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
