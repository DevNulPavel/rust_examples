use crate::network::Instance;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

const HTTP_ERROR_PROXYING_TUNNEL_RESPONSE: &'static [u8] =
    b"HTTP/1.1 500\r\nContent-Length: 28\r\n\r\nError: Error proxying tunnel";

pub async fn proxy_stream(instance: Instance, mut stream: TcpStream) {
    // Адрес инстанса и порт
    let addr = SocketAddr::new(instance.ip, crate::CONFIG.remote_port);

    // Подключаемся по адресу сокета
    let mut instance = match TcpStream::connect(addr).await {
        Ok(stream) => {
            stream
        },
        Err(error) => {
            tracing::error!(?error, "Error connecting to instance");
            // При ошибке пишем в выходной стрим ошибку подключения
            let _ = stream
                .write_all(HTTP_ERROR_PROXYING_TUNNEL_RESPONSE).await;
            return;
        }
    };

    // Потоки удаленного сервера
    let (mut i_read, mut i_write) = instance.split();
    // Наш поток
    let (mut r_read, mut r_write) = stream.split();

    // Копируем потоки из одного в другой
    let _ = futures::future::join(tokio::io::copy(&mut r_read, &mut i_write),
                                  tokio::io::copy(&mut i_read, &mut r_write))
        .await;
}
