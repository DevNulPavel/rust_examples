use eyre::WrapErr;
use futures::TryStreamExt;
use hyper::{
    body::Bytes,
    header,
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body as BodyStruct, Method, Request, Response, Server, StatusCode,
};
use std::{convert::Infallible, net::SocketAddr, process::exit};
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
                let resp = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(BodyStruct::empty())
                    .expect("Static fail response create failed"); // Статически создаем ответ, здесь не критично

                // Выходим с ошибкой
                return Ok(resp);
            }
        }
    };
    ($code: expr, $desc: literal) => {
        match $code.wrap_err($desc) {
            Ok(val) => val,
            Err(err) => {
                // Выводим ошибку
                error!("{}", err);
                // error!(concat!($desc, ", Err: {:?}"), err);

                // Создаем ответ с правильным статусом
                let resp = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(BodyStruct::empty())
                    .expect("Static fail response create failed"); // Статически создаем ответ, здесь не критично

                // Выходим с ошибкой
                return Ok(resp);
            }
        }
    };
}

#[instrument(level = "error")]
async fn service_handler(remove_addr: SocketAddr, req: Request<BodyStruct>) -> Result<Response<BodyStruct>, Infallible> {
    // debug!("Request processing begin");

    match (req.method(), req.uri().path()) {
        // Запрос к корню
        (&Method::GET, "/") => {
            info!("Root");
            let response = unwrap_or_fail_resp!(Response::builder()
                .status(StatusCode::MOVED_PERMANENTLY)
                .header(header::LOCATION, "/help")
                .body(BodyStruct::empty()));

            Ok(response)
        }

        // Помощь
        (&Method::GET, "/help") => {
            info!("Help");
            let response = unwrap_or_fail_resp!(Response::builder()
                .status(StatusCode::OK)
                .body(BodyStruct::from("Try to send POST request at '/echo'")));
            Ok(response)
        }

        // Возвращаем все в ответ
        (&Method::POST, "/echo") => {
            info!("Echo");
            let resp = unwrap_or_fail_resp!(Response::builder().status(StatusCode::OK).body(req.into_body()));
            Ok(resp)
        }

        // Возвращаем все в ответ в верхнем регистре
        (&Method::POST, "/echo/uppercase") => {
            info!("Echo uppercase");

            let new_stream = req.into_body().map_ok(|data_chunk| {
                let result: Bytes = data_chunk.into_iter().map(|val| val.to_ascii_uppercase()).collect();
                result
            });

            let resp = unwrap_or_fail_resp!(Response::builder().status(StatusCode::OK).body(BodyStruct::wrap_stream(new_stream)));
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

async fn execute_app() -> Result<(), eyre::Error> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    // Сервис необходим для каждого соединения, поэтому создаем враппер, который будет генерировать наш сервис
    let make_svc = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr();
        async move {
            // Создаем сервис из функции с помощью service_fn
            Ok::<_, Infallible>(service_fn(move |req| service_handler(remote_addr, req)))
        }
    });

    // Создаем сервер
    let server = Server::bind(&addr).serve(make_svc);

    // Запускаем сервер на постоянную работу
    server.await.wrap_err("Server awaiting")?;

    // TODO: Gracefull shutdown

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
