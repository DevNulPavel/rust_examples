use crate::{
    buffer_pool::BufferPool, config::Directive, directive::directive_process, error::CbltError,
    request::BUF_SIZE,
};
use log::{error, info};
use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;
use std::{collections::HashMap, sync::Arc};
use tokio::{net::TcpListener, sync::Semaphore};
use tokio_rustls::TlsAcceptor;

////////////////////////////////////////////////////////////////////////////////

/// Данные про сертификат для TLS
#[derive(Debug, Clone)]
pub(super) struct Cert {
    /// Путь к файлику сертификата
    pub(super) cert_path: PathBuf,

    /// Путь к файлику с ключем
    pub(super) key_path: PathBuf,
}

////////////////////////////////////////////////////////////////////////////////

/// Информация о сервере после конфигурирования
#[derive(Debug, Clone)]
pub(super) struct ServerConfig {
    /// Для какого порта обработка
    pub(super) port: u16,

    /// Директивы для обработки определенных хостов
    pub(super) hosts: HashMap<String, Vec<Directive>>, // Host -> Directives

    /// Данные для TLS сертификата
    pub(super) cert: Option<Cert>,
}

////////////////////////////////////////////////////////////////////////////////

/// Запускаем в работу "сервер" обработки
pub(super) async fn server_init(
    server: ServerConfig,
    max_connections: usize,
) -> Result<(), CbltError> {
    // Есть ли необходимость здесь у нас в TLS?
    let tls_acceptor = if let Some(cert) = server.cert.as_ref() {
        // TODO: Можно было бы здесь использовать асинхронную подгрузку?
        // Выполняем синхронную подгрузку сертификатов из указанного файлика,
        // отваливаемся в случае какой-то ошибки
        let certificates =
            CertificateDer::pem_file_iter(&cert.cert_path)?.collect::<Result<Vec<_>, _>>()?;

        // TODO: Аналогично - здесь можно было бы асинхронно подгружать?
        // Закрытый ключ для данных сертификатов
        let key = PrivateKeyDer::from_pem_file(&cert.key_path)?;

        // Создаем теперь конфиг для обработки входящих подключений
        let server_config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certificates, key)?;

        // Создаем теперь для указанного конфига возможность получать асинхронно новые подключения
        let acceptor = TlsAcceptor::from(Arc::new(server_config));

        Some(acceptor)
    } else {
        None
    };

    // Для ограничений максимального количества подключений будем использовать семафор
    let semaphore = Arc::new(Semaphore::new(max_connections));

    // Создаем листнер входящих подключений
    let listener = {
        // Формируем адрес для прослушивания как `0.0.0.0:<port>`
        let addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, server.port);

        // Создаем теперь с этим адресом асинхронный получатель подключения нового
        TcpListener::bind(addr).await?
    };
    info!("Listening port: {}", server.port);

    // Создаем пул буфферов сразу нужного размера для максимального количества подключений
    let buffer_pool = Arc::new(BufferPool::new(max_connections, BUF_SIZE));

    // Для прочих запросов создаем уже клиента reqwest
    let client_reqwest = reqwest::Client::new();

    loop {
        // Создаем клоны переменных для отдельной корутины
        let client_reqwest = client_reqwest.clone();
        let buffer_pool_arc = buffer_pool.clone();
        let tls_acceptor_clone = tls_acceptor.clone();
        let server_clone = server.clone();

        // Ждем очередное подключение какое-то на вход
        let mut stream = listener.accept().await?.0;

        // После получения очередного подключения ограничиваем максимальное
        // количество подключений входящих с помощью семафора.
        // Будем держать пермишен семафора до момента завершения
        // работы с входящим стримом.
        let permit = semaphore.clone().acquire_owned().await?;

        // Запускаем обработку сокета в отдельной корутине уже
        tokio::spawn(async move {
            // Получаем очередной буфер из пула сразу для возможности использования
            // здесь его.
            let mut buffer = buffer_pool_arc.get_buffer().await;

            // Есть ли у нас здесь необходимость в TLS?
            match tls_acceptor_clone.as_ref() {
                // Нужен нам TLS здесь
                Some(tls_acceptor) => {
                    // Пробуем получить уже TLS стрим
                    // для прилетевшего на вход IO стрима
                    match tls_acceptor.accept(stream).await {
                        // Все хорошо, получили TLS соединение
                        Ok(mut stream) => {
                            // Теперь выполняем непосредственно обработку уже
                            // над TLS стримом
                            let process_result = directive_process(
                                &mut stream,
                                &server_clone,
                                &mut buffer,
                                &client_reqwest,
                            )
                            .await;

                            // Если при обоработке возникли проблемы, тогда
                            // здесь просто нам остается написать про это
                            if let Err(err) = process_result {
                                error!("Processing error: {}", err);
                            }
                        }
                        // Ошибка какая-то в TLS
                        Err(err) => {
                            error!("TLS error: {}", err);
                        }
                    }
                }
                // Здесь нам не нужен TLS
                None => {
                    // Обрабатываем стрим уже как есть
                    let process_result =
                        directive_process(&mut stream, &server_clone, &mut buffer, &client_reqwest)
                            .await;

                    // Ошибка
                    if let Err(err) = process_result {
                        error!("Direct processing error: {}", err);
                    }
                }
            }

            // Очищаем буфер
            buffer.clear();
            // buffer.lock().await.clear();

            // Делаем возврат буфера в пул назад
            buffer_pool_arc.return_buffer(buffer).await;

            // Здесь явно уничтожаем пермишен.
            // За счет этого кода пермишен явно перемещается в футуру текущую.
            drop(permit);
        });
    }
}
