mod dynamodb;

use crate::dynamodb::test_dynamo_db;
use tracing::error;
use tracing_error::ErrorLayer;
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
    if let Err(err) = test_dynamo_db().await {
        error!("{:?}", err);
    }
}
