use eyre::Context;
use rusoto_core::Region;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, ListTablesInput};
use tracing::{error, info, instrument};
use tracing_error::ErrorLayer;
// use tracing_futures::Instrument;
use tracing_log::LogTracer;
use tracing_subscriber::prelude::*;

///////////////////////////////////////////////////////////////////////////////////////////

fn initialize_logs() {
    // Фильтрация на основе настроек
    let filter = tracing_subscriber::EnvFilter::from_default_env();

    // Логи в stdout
    let stdoud_sub = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

    // Error layer для формирования слоя ошибки
    let error_layer = ErrorLayer::default();

    // Суммарный обработчик
    let full_subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(error_layer)
        .with(stdoud_sub);

    // Враппер для библиотеки log
    LogTracer::init().expect("Log wrapper create failed");

    // Установка по-умолчанию
    tracing::subscriber::set_global_default(full_subscriber).unwrap();
}

#[instrument]
async fn test_dynamodb() -> Result<(), eyre::Error> {
    let client = DynamoDbClient::new(Region::UsEast1);
    let list_tables_input: ListTablesInput = Default::default();

    let output = client
        .list_tables(list_tables_input)
        .await
        .context("Tables list request")?;

    match output.table_names {
        Some(table_name_list) => {
            info!(tables = ?table_name_list, "Tables in database");
        }
        None => {
            info!("No tables in database!")
        }
    }

    Ok(())
}


#[tokio::main]
async fn main() {
    // Настройка поддержки бектрейсов в ошибках
    color_eyre::install().expect("Backtrace setup failed");

    // Read env from .env file
    dotenv::from_filename("env/test.env").ok();

    // Friendly panic messages
    human_panic::setup_panic!();

    // Logs
    initialize_logs();

    // Test DynamoDB
    if let Err(err) = test_dynamodb().await {
        error!("Dynamo error -> {:?}", err);
    }
}
