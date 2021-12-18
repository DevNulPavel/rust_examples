mod app_arguments;

use crate::app_arguments::AppArguments;
use eyre::WrapErr;
use mongodb::{options::ClientOptions, Client};
use structopt::StructOpt;
use tracing::debug;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn initialize_logs() -> Result<(), eyre::Error> {
    use tracing_subscriber::prelude::*;

    /*let level = match arguments.verbose {
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
    let filter = tracing_subscriber::filter::LevelFilter::from_level(level);*/

    // Фильтрация на основе окружения
    let filter = tracing_subscriber::filter::EnvFilter::from_default_env();

    // Логи в stdout
    let stdoud_sub = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

    // Error layer для формирования слоя ошибки по запросу
    let error_layer = tracing_error::ErrorLayer::default();

    // Специальный слой для отладочной консоли tokio
    // Используем стандартные настройки для фильтрации из переменной RUST_LOG
    // let console_layer = console_subscriber::ConsoleLayer::builder().with_default_env().spawn();

    // Суммарный обработчик
    let full_subscriber = tracing_subscriber::registry()
        // .with(console_layer)
        .with(filter)
        .with(error_layer)
        .with(stdoud_sub);

    // Враппер для библиотеки log
    tracing_log::LogTracer::init().wrap_err("Log wrapper create failed")?;

    // Установка по-умолчанию
    tracing::subscriber::set_global_default(full_subscriber).wrap_err("Global subscriber set failed")?;

    Ok(())
}

async fn build_mongo_client(arguments: &AppArguments) -> Result<Client, eyre::Error> {
    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse(&arguments.mongodb_connection_addr)
        .await
        .wrap_err("Mongo connection string parsing")?;

    // Manually set an option.
    client_options.app_name = Some("Learning application".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options).wrap_err("Mongo client building")?;

    Ok(client)
}

async fn run_app(arguments: AppArguments) -> Result<(), eyre::Error> {
    let client = build_mongo_client(&arguments).await.wrap_err("Client building")?;

    // List the names of the databases in that deployment.
    for db_name in client.list_database_names(None, None).await? {
        println!("{}", db_name);
    }

    Ok(())
}

fn main() {
    // Бектрейсы в ошибках
    color_eyre::install().expect("Color eyre initialize failed");

    // Логи
    initialize_logs().expect("Logs init");

    // Парсим параметры приложения
    let arguments = AppArguments::from_args_safe().expect("App arguments parsing");
    debug!("App arguments: {:?}", arguments);

    // Создаем рантайм для работы
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Tokio runtime build");

    // Стартуем сервер
    runtime.block_on(run_app(arguments)).expect("Server running fail");
}
