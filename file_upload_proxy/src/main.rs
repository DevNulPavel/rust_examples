mod app_arguments;

use eyre::WrapErr;
use tracing::{
    instrument,
    error,
    debug,
    info,
    warn,
};
use futures::{future::pending};
use structopt::{
    StructOpt
};
use std::{
    net::{
        SocketAddr
    },
    convert::{
        Infallible
    }
};
use tokio::{
    sync::{
        broadcast,
        mpsc
    }
};
use hyper::{
    service::{
        make_service_fn,
        service_fn
    },
    server::{conn::AddrStream, Server},
};
use crate::{
    app_arguments::{
        AppArguments
    }
};

fn initialize_logs(arguments: &AppArguments) {
    let level = match arguments.verbose {
        0 => tracing::Level::ERROR,
        1 => tracing::Level::WARN,
        2 => tracing::Level::INFO,
        3 => tracing::Level::DEBUG,
        4 => tracing::Level::TRACE,
        _ => {
            panic!("Verbose level must be in [0, 4] range");
        }
    };

    // Фильтрация на основе настроек
    let filter = tracing_subscriber::filter::LevelFilter::from_level(level);

    // Логи в stdout
    let stdoud_sub = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

    // Error layer для формирования слоя ошибки
    let error_layer = tracing_error::ErrorLayer::default();

    // Суммарный обработчик
    let full_subscriber = tracing_subscriber::registry().with(filter).with(error_layer).with(stdoud_sub);

    // Враппер для библиотеки log
    tracing_log::LogTracer::init().expect("Log wrapper create failed");

    // Установка по-умолчанию
    tracing::subscriber::set_global_default(full_subscriber).expect("Global subscriber set failed");
}

#[instrument(level = "error", skip(cancel))]
async fn service_handler(
    remove_addr: SocketAddr,
    req: Request<BodyStruct>,
    cancel: Cancelation,
) -> Result<Response<BodyStruct>, eyre::Error> {
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
        (&Method::GET, "/web_socket") => process_web_socket(req, cancel).await,

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

async fn run_server(app_arguments: AppArguments) -> Result<(), eyre::Error> {
    let addr = SocketAddr::from(([0, 0, 0, 0], app_arguments.port));

    // Канал для отправки всем завершения работы
    let (finish_sender, finish_receiver) = broadcast::channel::<mpsc::Sender<()>>(1);
    drop(finish_receiver);

    // Сервис необходим для каждого соединения, поэтому создаем враппер, который будет генерировать наш сервис
    let make_svc = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr();
        let sender = finish_sender.clone();
        async move {
            // Создаем сервис из функции с помощью service_fn
            Ok::<_, Infallible>(service_fn(move |req| {
                let receiver = sender.subscribe();
                async move {
                    match service_handler(remote_addr, req, finish_receiver).await {
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
                }
            }))
        }
    });

    // Создаем сервер c ожиданием завершения работы
    Server::bind(&addr)
        .serve(make_svc)
        .with_graceful_shutdown(shutdown_signal_wait())
        .await
        .wrap_err("Server awaiting fail")?;

    // Ждем здесь завершения всех заспавненых обработчиков вебсокетов
    // Создаем канал для ожидания завершения, как только все отправители уничтожены -
    // получатель возвращает ошибку, но мы ее игнорим.
    // Некий аналог WaitGroup
    // https://tokio.rs/tokio/topics/shutdown
    let (wait_sender, mut wait_receiver) = mpsc::channel(1);
    finish_sender.send(wait_sender).wrap_err("Awaiters complete send err")?;
    let _ = wait_receiver.recv().await;
    info!("All clients processing complete");

    Ok(())
}

fn main() {
    // Бектрейсы в ошибках
    color_eyre::install().expect("Color eyre initialize failed");
    
    // Аргументы приложения
    let app_arguments = AppArguments::from_args();

    // Логи
    initialize_logs(&app_arguments);

    // Создаем рантайм для работы сервера
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Tokio runtime build");

    // Стартуем сервер
    runtime.block_on(run_server(app_arguments)).expect("Server running fail");
}
