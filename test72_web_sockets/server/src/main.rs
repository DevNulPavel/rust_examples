mod error;
mod macroses;

use crate::error::{ErrorWithStatusAndDesc, WrapErrorWithStatusAndDesc};
use eyre::WrapErr;
use futures::{future::pending, TryStreamExt};
use futures::{SinkExt, StreamExt};
use hyper::{
    body::to_bytes,
    body::Bytes,
    header::{self, HeaderName},
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    upgrade::Upgraded,
    Body as BodyStruct, Method, Request, Response, Server, StatusCode,
};
use std::{convert::Infallible, net::SocketAddr, process::exit};
use tokio::io::BufStream;
use tokio_tungstenite::tungstenite::{
    error::ProtocolError,
    protocol::{Message, Role},
    Error as WsError,
};
use tracing::{debug, error, info, instrument};
use tracing_error::ErrorLayer;
use tracing_futures::Instrument;
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

fn get_header_str<'a>(req: &'a Request<BodyStruct>, header: &HeaderName) -> Result<&'a str, eyre::Error> {
    let header_val = req
        .headers()
        .get(header)
        .ok_or_else(|| ErrorWithStatusAndDesc::new(StatusCode::BAD_REQUEST, format!("{} header is missing", header.as_str()).into()))?
        .to_str()
        .wrap_err_with_status_fn_desc(StatusCode::BAD_REQUEST, || {
            format!("Invalid {} header value", header.as_str()).into()
        })?;
    Ok(header_val)
}

/*fn get_optional_header_str<'a>(req: &'a Request<BodyStruct>, header: &HeaderName) -> Result<Option<&'a str>, eyre::Error> {
    if let Some(header_val) = req.headers().get(header) {
        let res = header_val.to_str().wrap_err_with_status_fn_desc(StatusCode::BAD_REQUEST, || {
            format!("Invalid {} header value", header.as_str()).into()
        })?;
        Ok(Some(res))
    } else {
        Ok(None)
    }
}*/

#[instrument(level = "error", skip(upgraded))]
async fn web_socket_io(upgraded: Upgraded) -> Result<(), eyre::Error> {
    // Оборачеваем данный поток в фуффер, чтобы не надо было делать на каждое чтение системный вызов
    let mut ws = tokio_tungstenite::WebSocketStream::from_raw_socket(BufStream::new(upgraded), Role::Server, None).await;
    while let Some(message) = ws.next().await {
        debug!(?message);

        // TODO: При остановке сервера отсылать всем клиентам CLOSE

        // Разворачиваем вообщение
        match message {
            Ok(Message::Text(received_text)) => {
                // Если пришло - значит завершаем работу с сокетом
                if received_text == "stop" {
                    ws.close(None).await.wrap_err("Close send")?;
                    drop(ws);
                    break;
                }

                // Отправляем назад
                ws.send(Message::Text(received_text.to_uppercase())).await.wrap_err("WS write")?;
            }
            Ok(Message::Binary(_)) => {
                error!("Binary data unsupported");
                ws.close(None).await.wrap_err("Close send")?;
                drop(ws);
                break;
            }
            Ok(Message::Close(_)) => {
                debug!("Socket closed normaly");
                break;
            }
            Ok(Message::Ping(data)) => {
                debug!("Send pong");
                ws.send(Message::Pong(data)).await.wrap_err("WS write")?;
                continue;
            }
            Ok(Message::Pong(_)) => {
                error!("Client must send ping");
                break;
            }
            Err(err) => {
                // Обработаем тип ошибки
                match err {
                    // Ошибка протокола, не закрыли нормально
                    WsError::Protocol(ProtocolError::ResetWithoutClosingHandshake) => {
                        warn!("Connection closed without valid message");
                        break;
                    }
                    _ => {
                        return Err(err.into());
                    }
                }
            }
        }
    }
    info!("Socket processing complete");

    Ok(())
}

#[instrument(level = "error", skip(req))]
async fn process_web_socket(mut req: Request<BodyStruct>) -> Result<Response<BodyStruct>, eyre::Error> {
    info!("Web socket");

    // Проверим, что в заголовках есть заголовок UPGRADE, либо выходим с ошибкой
    let upgrade_header_val = get_header_str(&req, &header::UPGRADE)?;
    // let connection_header_val = get_header_str(&req, &header::CONNECTION)?;
    let ws_key_header_val = get_header_str(&req, &header::SEC_WEBSOCKET_KEY)?; // Опциональный заголовок, но будем его требовать
    let ws_version_header_val = get_header_str(&req, &header::SEC_WEBSOCKET_VERSION)?;
    // let ws_protocols_header_val = get_optional_header_str(&req, &header::SEC_WEBSOCKET_PROTOCOL)?;

    // debug!(?connection_header_val);
    // debug!(?ws_protocols_header_val);

    // Проверить версию, если версия не подходящая, тогда ответить нужной версией в заголовке header::SEC_WEBSOCKET_VERSION
    // и статусом 426 Upgrade Required
    // Текущая версия - 13
    if ws_version_header_val != "13" {
        return Ok(response_with_status_end_error(
            StatusCode::UPGRADE_REQUIRED,
            "Invalid web socket version, must be 13",
        ));
    }

    // Проверить WebSocket ключ, на основании данного ключа выдаем значение в заголовок Sec-WebSocket-Accept ответа
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Protocol_upgrade_mechanism#response-only_headers
    let accept_key = {
        let accept_key_source = format!("{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11", ws_key_header_val);
        use sha1::Digest;
        let sha1_val = sha1::Sha1::digest(accept_key_source.as_bytes());
        base64::encode(sha1_val)
    };

    // Сразу формируем ответ
    let result_response = Response::builder()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .header(header::UPGRADE, upgrade_header_val)
        .header(header::SEC_WEBSOCKET_ACCEPT, accept_key)
        .body(BodyStruct::empty())?;
    debug!(?result_response);

    // Создаем футуру, которая в конце концов получит обновленное соединение и будет работать с ним
    // Эта трансформация не может быть выполнена до тех пор, пока мы не ответим 101 кодом,
    // Но футуру лучше запускать заранее, чтобы висеть на ней.
    // TODO: Дополнительно дождемся старта футуры или клиент все равно не пойдет дальше?
    tokio::task::spawn(
        async move {
            match hyper::upgrade::on(&mut req).await {
                Ok(upgraded) => {
                    if let Err(e) = web_socket_io(upgraded).await {
                        error!("Server web socket io error: {:?}", e)
                    };
                }
                Err(e) => error!("Upgrade error: {:?}", e),
            }
        }
        .instrument(tracing::span!(tracing::Level::ERROR, "ws_upgrade_processing")), // Для правильного продолжения активного Span внутри вызова span
    );

    Ok(result_response)
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
                .wrap_err_with_status(StatusCode::BAD_GATEWAY, "Body to bytes convert failed".into())?;

            let rev_body_data: Bytes = body_data.iter().rev().cloned().collect();

            let resp = Response::builder()
                .status(StatusCode::OK)
                .body(BodyStruct::from(rev_body_data))
                .wrap_err("Body build")?;
            Ok(resp)
        }

        // Возвращаем все в ответ в обратном порядке
        (&Method::GET, "/web_socket") => process_web_socket(req).await,

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
