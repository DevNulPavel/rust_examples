use eyre::WrapErr;
use std::{
    fs::File,
    io::Read,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::Path,
    sync::Arc,
};
use tracing::debug;

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

fn read_all_file(path: &Path) -> Result<Vec<u8>, eyre::Error> {
    let mut cert_file = File::open(path).wrap_err("Certificate file open")?;
    let mut buf = Vec::new();
    cert_file.read_to_end(&mut buf).wrap_err("File read")?;
    Ok(buf)
}

async fn execute_app() -> Result<(), eyre::Error> {
    // Настройка логирования на основании количества флагов verbose
    setup_logging().wrap_err("Logging setup")?;

    // Создаем пустое хранилище для корневых сертификатов
    let root_certificates_store = {
        let mut root_certificates_store = rustls::RootCertStore::empty();
        // Без сертификата это все не будет работать, поэтому падаем с ошибкой
        let cert_data = read_all_file(Path::new("tmp_certificates/cert.der")).wrap_err("Certificate file read")?;
        root_certificates_store
            .add(&rustls::Certificate(cert_data))
            .wrap_err("Failed to add root certificate")?;
        root_certificates_store
    };

    // Клиентский конфиг криптографии
    let mut client_crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_certificates_store)
        .with_no_client_auth();
    // Указываем поддерживаемые протоколы в порядке убывания приоритетности
    client_crypto.alpn_protocols = vec![b"hq-29".to_vec()];
    // Настраиваем логирование
    client_crypto.key_log = Arc::new(rustls::KeyLogFile::new());

    // В качестве локального адреса приема пакетов указываем произвольное значение
    let receive_address = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0));
    let mut endpoint = quinn::Endpoint::client(receive_address).wrap_err("Bind client failed")?;
    endpoint.set_default_client_config(quinn::ClientConfig::new(Arc::new(client_crypto)));

    // Устанавливаем соединение
    let quinn::NewConnection { connection: conn, .. } = endpoint
        .connect(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8443)), "localhost")?
        .await
        .wrap_err("Connection to server failed")?;

    // Открываем по открытому соединению новыйз запрос и получаем каналы для отправки и получения данных
    let (mut send, recv) = conn.open_bi().await.wrap_err("Request open failed")?;

    send.write_all(b"This is the test data").await.wrap_err("Request send failed")?;
    send.finish().await.wrap_err("Send finish failed")?;

    // Получаем ответ и выводим его
    let resp = recv.read_to_end(128 * 1024).await.wrap_err("Reponce receive")?;

    let response_text = std::str::from_utf8(resp.as_slice()).wrap_err("Responce parse failed")?;
    debug!("Server response: {}", response_text);

    // Завершаем работу с соединением
    debug!("Gracefull client shutdown");
    endpoint.wait_idle().await;

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
