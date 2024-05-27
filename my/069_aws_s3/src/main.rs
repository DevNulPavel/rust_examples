use eyre::Context;
use rusoto_core::Region;
use rusoto_s3::{CreateBucketRequest, ListObjectsRequest, S3Client, S3};
use std::process::exit;
use tracing::error;
use tracing_error::ErrorLayer;
use tracing_log::LogTracer;
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

async fn execute_app() -> Result<(), eyre::Error> {
    // Создаем клиент S3
    let s3_client = S3Client::new(Region::EuWest2);

    // Список файликов в корзине
    {
        let mut list_request = ListObjectsRequest::default();
        list_request.bucket = "public-files-bucket".to_string();

        let result = s3_client.list_objects(list_request).await?;
        println!("Files list: {:?}", result);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // Бектрейсы в ошибках
    color_eyre::install().unwrap();

    // Логи
    initialize_logs().unwrap();

    // Читаем переменные окружения из файлика
    dotenv::from_filename("env/test.env").unwrap();

    if let Err(err) = execute_app().await {
        error!("{:?}", err);
        exit(1);
    } else {
        exit(0);
    }
}
