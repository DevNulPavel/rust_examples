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
pub(super) struct Server {
    /// Для какого порта обработка
    pub(super) port: u16,

    /// Директивы для обработки определенных хостов
    pub(super) hosts: HashMap<String, Vec<Directive>>, // Host -> Directives

    /// Данные для TLS сертификата
    pub(super) cert: Option<Cert>,
}

////////////////////////////////////////////////////////////////////////////////

/// Запускаем в работу "сервер" обработки
pub(super) async fn server_init(server: Server, max_connections: usize) -> Result<(), CbltError> {
    // Есть ли необходимость здесь у нас в TLS?
    let tls_acceptor = if let Some(cert) = server.cert {
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

    let client_reqwest = reqwest::Client::new();

    loop {
        let client_reqwest = client_reqwest.clone();
        let buffer_pool_arc = buffer_pool.clone();
        let acceptor_clone = tls_acceptor.clone();
        let server_clone = server.clone();
        let (mut stream, _) = listener.accept().await?;
        let permit = semaphore.clone().acquire_owned().await?;
        tokio::spawn(async move {
            let _permit = permit;
            let buffer = buffer_pool_arc.get_buffer().await;
            match acceptor_clone {
                None => {
                    if let Err(err) = directive_process(
                        &mut stream,
                        &server_clone,
                        buffer.clone(),
                        client_reqwest.clone(),
                    )
                    .await
                    {
                        error!("Error: {}", err);
                    }
                }
                Some(ref acceptor) => match acceptor.accept(stream).await {
                    Ok(mut stream) => {
                        if let Err(err) = directive_process(
                            &mut stream,
                            &server_clone,
                            buffer.clone(),
                            client_reqwest.clone(),
                        )
                        .await
                        {
                            error!("Error: {}", err);
                        }
                    }
                    Err(err) => {
                        error!("Error: {}", err);
                    }
                },
            }
            buffer.lock().await.clear();
            buffer_pool_arc.return_buffer(buffer).await;
        });
    }
}
