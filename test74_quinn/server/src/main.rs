mod sertificate_gen;

use crate::sertificate_gen::generate_https_sertificate;
use eyre::{ContextCompat, WrapErr};
use futures::StreamExt;
use log::{debug, LevelFilter};
use quinn::ServerConfig;
use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Настойка уровня логирования
fn setup_logging() -> Result<(), eyre::Error> {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(LevelFilter::Trace)
        .try_init()?;
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

async fn execute_app() -> Result<(), eyre::Error> {
    // Настройка логирования на основании количества флагов verbose
    setup_logging().wrap_err("Logging setup")?;

    // Создаем конфиг для нашего сервера
    let server_config = make_server_config().wrap_err("Server config")?;

    // Сервер
    let listen_address = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8443));
    let (endpoint, mut incoming) = quinn::Endpoint::server(server_config, listen_address)?;
    debug!("Listening on: {}", endpoint.local_addr()?);

    // Цикл принятия новых соединений
    while let Some(conn) = incoming.next().await {
        debug!(
            "Connection accepted: local_ip = {:?}, remote_addr = {:?}",
            conn.local_ip(),
            conn.remote_address()
        );
        // tokio::spawn(
        // handle_connection(root.clone(), conn).unwrap_or_else(move |e| {
        //     error!("connection failed: {reason}", reason = e.to_string())
        // }),
        // );
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
