mod sertificate_gen;

use crate::sertificate_gen::generate_https_sertificate;
use eyre::{ContextCompat, WrapErr};
use futures::StreamExt;
use quinn::ServerConfig;
use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
};
use tracing::{debug, error, error_span, info, Instrument};
use tracing_log::log::warn;

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

fn debug_print_handshake_info(connection: &quinn::Connection) {
    let handshake_data = connection
        .handshake_data()
        .and_then(|data| data.downcast::<quinn::crypto::rustls::HandshakeData>().ok());
    if let Some(data) = handshake_data {
        debug!("Handshake data: server = {:?}, protocol = {:?}", data.server_name, data.protocol);
    } else {
        warn!("Empty handshake data");
    }
}

async fn process_request(mut send_stream: quinn::SendStream, recv_stream: quinn::RecvStream, request_id: u128) -> Result<(), eyre::Error> {
    // Читаем все данные до закрытия соединения в рамках этого запроса
    let req_data = recv_stream
        .read_to_end(64 * 1024) // 64kB максимальный размер входного буффера
        .await
        .wrap_err("Read failed")?;

    // Парсим UTF-8
    // Можно воспользоваться вызовом req_data.to_ascii_uppercase();
    // Но мы парсим текст просто для вызова вывода в логи
    let req_text = std::str::from_utf8(req_data.as_slice()).wrap_err("Request is not UTF-8")?;
    info!("Request: {}", req_text);

    // Конвертируем в верхний регистр
    let response_text = format!(r#"{{"request_id": {}, "response": "{}"}}"#, request_id, req_text.to_uppercase());
    drop(req_text);

    // Пишем ответ на наш запрос
    send_stream.write_all(response_text.as_bytes()).await.wrap_err("Response send")?;

    // Gracefully terminate the stream
    send_stream.finish().await.wrap_err("Gracefull stream finish failed")?;

    info!("Complete");

    Ok(())
}

/// Непосредственно обработка пришедшего соединения
async fn process_connection(_: quinn::Connection, mut bi_streams: quinn::IncomingBiStreams) -> Result<(), eyre::Error> {
    debug!("Connection processing started");

    // В рамках одного подключения может быть несколько разных подключений
    while let Some(received_streams) = bi_streams.next().await {
        match received_streams {
            // Все хорошо, разворачиваем в отдельный стрим отправки данных и получения
            Ok((req_send_stream, req_receive_stream)) => {
                // Перед обработкой конкретного запроса создадим span для конкретного запроса
                // Чтобы можно было удобно группировать запросы по конкретному request_id в логах
                let request_id = uuid::Uuid::new_v4().as_u128();
                let span = error_span!("request", %request_id);

                // Обрабатываем запрос в отдельной корутине
                tokio::spawn(
                    async move {
                        if let Err(err) = process_request(req_send_stream, req_receive_stream, request_id).await {
                            error!("Request processing error: {}", err);
                        }
                    }
                    .instrument(span),
                );
            }

            // Обработка закрытия подключения в целом
            Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                debug!("connection closed");
                return Ok(());
            }

            // Прочие ошибки
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    Ok(())
}

/// Непосредственно обработка пришедшего соединения с установкой handshake
async fn process_accepted_connection(accepted_conn: quinn::Connecting) -> Result<(), eyre::Error> {
    debug!("Connection processing started");

    // TODO: посмотреть на вызовы HADNSHAKE

    // Дожидаемся установки соединения полноценного, разных хендшейков и тд
    let quinn::NewConnection {
        connection, bi_streams, ..
    } = accepted_conn.await.wrap_err("Connection establish")?;

    // Для трассировки создаем span, для группировки используем теперь id после handshake
    let handshake_span = tracing::error_span!("handshake", id = %connection.stable_id());

    // Сразу выведем служебную информацию про span перед началом обработки
    {
        let _guard = handshake_span.enter();

        // Выводим данные о HTTPS handshake
        debug_print_handshake_info(&connection);

        // Выводим прочую информацию о соединении
        debug!(
            "Connection best RTT: {}ms, QUIC connection id: {}",
            connection.rtt().as_millis(),
            connection.stable_id()
        );
    }

    // Запускаем обработку
    process_connection(connection, bi_streams).instrument(handshake_span).await
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
        let span = tracing::error_span!("connection", remote_addr = ?accepted_conn.remote_address());

        // Залогируем более подробную информацию данном соединении
        // Но только синхронные быстрые вызовы, чтобы не блокировать получение новых подключений
        {
            let _span_enter = span.enter();
            let local_ip = accepted_conn.local_ip();
            let remote_ip = accepted_conn.remote_address();

            debug!("Connection accepted: local_ip = {:?}, remote_addr = {:?}", local_ip, remote_ip,);
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
