mod app_arguments;

use crate::app_arguments::AppArguments;
use eyre::WrapErr;
use futures::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{doc, Bson},
    options::ClientOptions,
    options::FindOptions,
    Client, Database,
};
use serde::{Deserialize, Serialize};
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
    // Парсим строку, описывающее соединение
    let mut client_options = ClientOptions::parse(&arguments.mongodb_connection_addr)
        .await
        .wrap_err("Mongo connection string parsing")?;

    // Дополнительно выставляем имя приложения
    client_options.app_name = Some("Learning application".to_string());

    // Создаем непосредственно клиента
    let client = Client::with_options(client_options).wrap_err("Mongo client building")?;

    Ok(client)
}

async fn print_databases(client: &Client) -> Result<(), eyre::Error> {
    for (i, db_name) in client
        .list_database_names(None, None)
        .await
        .wrap_err("Databases list")?
        .into_iter()
        .enumerate()
    {
        debug!("Database {}: {}", i, db_name);
    }
    Ok(())
}

async fn print_collections(db: &Database) -> Result<(), eyre::Error> {
    for (i, collection_name) in db
        .list_collection_names(None)
        .await
        .wrap_err("Collections list")?
        .into_iter()
        .enumerate()
    {
        debug!("Collection {}: {}", i, collection_name);
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Airline {
    id: Bson,
    name: String,
    alias: String,
    iata: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Route {
    airline: Airline,
    src_airport: String,
    dst_airport: String,
    codeshare: String,
    stops: Bson,
    airplane: Bson,
}

async fn run_app(arguments: AppArguments) -> Result<(), eyre::Error> {
    // Создаем клиента
    let client = build_mongo_client(&arguments).await.wrap_err("Client building")?;

    // Выводим список баз данных на данном соединении
    print_databases(&client).await.wrap_err("DB list")?;

    // Делаем подключение к базе с конкретным именем
    let db = client.database(&arguments.mongodb_database_name);

    // Span для открытой базы
    let _dbspan = tracing::error_span!("database", name = %arguments.mongodb_database_name).entered();

    // Выводим список коллекций
    print_collections(&db).await.wrap_err("Collections list")?;

    // Выбираем конкретную коллекцию
    let collection = db.collection::<Route>(&arguments.mongodb_collection_name);

    // Span для коллекции
    let _collection_span = tracing::error_span!("collection", name = %arguments.mongodb_collection_name).entered();

    // Делаем запрос данных с фильтрацией
    let filter = doc! { "src_airport": "MMK" };
    let find_options = FindOptions::builder().limit(10).sort(doc! { "airplane": 1 }).build();
    let mut cursor = collection.find(filter, find_options).await.wrap_err("Collection find")?;

    // Итерируемся по всем найденным результатам
    while let Some(route) = cursor.try_next().await.wrap_err("Cursor next")? {
        println!("Title: {:?}", route);
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
