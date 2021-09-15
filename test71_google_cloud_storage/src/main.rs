mod auth;
mod helpers;
mod types;

use crate::{
    auth::{get_token_data, ServiceAccountData},
    types::HttpClient,
};
use eyre::WrapErr;
use hyper::{body::Body as BodyStruct, Client};
use hyper_rustls::HttpsConnector;
use std::{path::Path, process::exit};
use tracing::{debug, error, info};
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

fn build_http_client() -> HttpClient {
    // Коннектор для работы уже с HTTPS
    let https_connector = HttpsConnector::with_native_roots();

    // Клиент с коннектором
    let http_client = Client::builder().set_host(false).build::<_, BodyStruct>(https_connector);

    http_client
}

async fn execute_app() -> Result<(), eyre::Error> {
    // Описание логики получения токена из сервисного аккаунта
    // https://developers.google.com/identity/protocols/oauth2/service-account#httprest
    // https://cloud.google.com/storage/docs/authentication

    // Данные по сервисному аккаунту
    let service_acc_data =
        ServiceAccountData::new_from_file(Path::new("./env/test_google_service_account.json")).wrap_err("Service account read")?;
    debug!("Service account data: {:?}", service_acc_data);

    // Клиент для http
    let http_client = build_http_client();

    // Получаем токен для работы
    const SCOPES: &str = "https://www.googleapis.com/auth/devstorage.read_write";
    let token_data = get_token_data(&http_client, &service_acc_data, SCOPES)
        .await
        .wrap_err("Token receive")?;
    info!("Received token: {:?}", token_data);

    Ok(())
}

#[tokio::main]
async fn main() {
    // Бектрейсы в ошибках
    color_eyre::install().unwrap();

    // Читаем переменные окружения из файлика
    dotenv::from_filename("env/test.env").unwrap();

    // Логи
    initialize_logs().unwrap();

    if let Err(err) = execute_app().await {
        error!("{:?}", err);
        exit(1);
    } else {
        exit(0);
    }
}
