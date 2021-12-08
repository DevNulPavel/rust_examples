mod sertificate_gen;

use crate::sertificate_gen::generate_https_sertificate;
use eyre::{ContextCompat, WrapErr};
use futures::StreamExt;
use quinn::ServerConfig;
use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
};
use tracing::{debug, error, instrument, Instrument};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Настойка уровня логирования
fn setup_logging() -> Result<(), eyre::Error> {
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

    // Предустановленное значение
    let filter = tracing_subscriber::filter::LevelFilter::from_level(tracing::Level::TRACE);

    // Фильтрация на основе окружения
    /*let filter = tracing_subscriber::filter::EnvFilter::from_default_env();*/

    // Логи в stdout
    let stdoud_sub = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

    // Error layer для формирования слоя ошибки
    let error_layer = tracing_error::ErrorLayer::default();

    // Суммарный обработчик
    let full_subscriber = tracing_subscriber::registry().with(filter).with(error_layer).with(stdoud_sub);

    // Враппер для библиотеки log
    tracing_log::LogTracer::init().wrap_err("Log wrapper create failed")?;

    // Установка по-умолчанию
    tracing::subscriber::set_global_default(full_subscriber).wrap_err("Global subscriber set failed")?;

    Ok(())
}

fn make_server_config() -> Result<ServerConfig, eyre::Error> {
    // Генерируем самоподписные сертификаты для HTTPS
    let certificate = generate_https_sertificate().wrap_err("Sertificate create")?;

    // Конфиг сервера для RUTLS
    let mut server_crypto = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![certificate.certificate], certificate.private_key)?;
    // Указываем поддерживаемые протоколы в порядке убывания приоритетности
    server_crypto.alpn_protocols = vec![b"hq-29".to_vec()];
    // Выводим лог криптографии в файлик
    server_crypto.key_log = Arc::new(rustls::KeyLogFile::new());

    // Конфиг для QUINN
    let mut server_config = quinn::ServerConfig::with_crypto(Arc::new(server_crypto));
    {
        let transport = Arc::get_mut(&mut server_config.transport).wrap_err("Mut transport ref")?;
        transport.max_concurrent_uni_streams(0_u8.into());
    }
    server_config.use_retry(true);

    Ok(server_config)
}

/// Непосредственно обработка пришедшего соединения
async fn process_accepted_connection(accepted_conn: quinn::Connecting) -> Result<(), eyre::Error> {
    debug!("Connection processing started");

    // Дожидаемся установки соединения полноценного, разных хендшейков и тд
    let new_conn = accepted_conn.await.wrap_err("Connection establish")?;

    Ok(())
}

async fn execute_app() -> Result<(), eyre::Error> {
    // Настройка логирования на основании количества флагов verbose
    setup_logging().wrap_err("Logging setup")?;

    // Создаем конфиг для нашего сервера
    let server_config = make_server_config().wrap_err("Server config")?;

    // Сервер
    let listen_address = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8443));
    let (endpoint, mut incoming) = quinn::Endpoint::server(server_config, listen_address)?;
    debug!("Listening on: {}", endpoint.local_addr()?);

    // Цикл принятия новых попыток соединений
    while let Some(accepted_conn) = incoming.next().await {
        // Для трассировки создаем span, для группировки используем удаленный адрес
        let span = tracing::error_span!("accept", remove_addr = ?accepted_conn.remote_address());

        // Залогируем более подробную информацию данном соединении
        {
            let _span_enter = span.enter();
            debug!(
                "Connection accepted: local_ip = {:?}, remote_addr = {:?}",
                accepted_conn.local_ip(),
                accepted_conn.remote_address(),
            );
        }

        // Запускаем асинхронную обработку задачи в контексте текущего span
        tokio::spawn(
            async move {
                if let Err(err) = process_accepted_connection(accepted_conn).await {
                    error!("Connection processing error: {}", err);
                }
            }
            .instrument(span),
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // Настройка color eyre для ошибок
    color_eyre::install().expect("Error setup failed");

    // Запуск приложения
    if let Err(err) = execute_app().await {
        // При ошибке не паникуем, а спокойно выводим сообщение и завершаем приложение с кодом ошибки
        eprint!("Error! Failed with: {:?}", err);
        std::process::exit(1);
    }
}
