use crate::strategy::{HostToServerMap, ProxyStrategy};
use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::io::{self, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;

/// TCP-проксирование
pub struct TcpProxy;

#[async_trait]
impl ProxyStrategy for TcpProxy {
    async fn run(&self, ct: CancellationToken, hs_map: HostToServerMap) -> io::Result<()> {
        for (host, server) in hs_map {
            log::info!("starting tcp proxy, host: {}, server: {}", host, server);

            let ln = TcpListener::bind(host).await?;
            tokio::spawn(incoming_connections_loop(ct.clone(), ln, server));
        }

        Ok(())
    }
}

/// Цикл приема входящих TCP-подключений
async fn incoming_connections_loop(ct: CancellationToken, ln: TcpListener, server: SocketAddr) {
    loop {
        tokio::select! {
            Ok((in_stream, in_addr)) = ln.accept() => {
                // На каждое подключение создаем таск обработки подключения
                tokio::spawn(process_incoming_connection(ct.clone(), in_stream, in_addr, server));
            }
            // `CancellationToken` отменен, больше не нужно принимать входящие соединения
            _ = ct.cancelled() => break
        }
    }
}

/// Обработка входящего TCP-соединения
async fn process_incoming_connection(
    ct: CancellationToken,
    mut in_stream: TcpStream,
    in_addr: SocketAddr,
    server: SocketAddr,
) {
    log::debug!("incoming tcp connection: {}", in_addr);

    let mut out_stream = match TcpStream::connect(server).await {
        Ok(s) => s,
        Err(err) => {
            log::error!("can't connect to server {}: {}", server, err);
            return;
        }
    };

    let (mut in_read, mut in_write) = in_stream.split();
    let (mut out_read, mut out_write) = out_stream.split();

    let client_to_server = async {
        tokio::select! {
            res = io::copy(&mut in_read, &mut out_write) => res.map(|_| ())?,
            _ = ct.cancelled() => {}
        }
        out_write.shutdown().await
    };

    let server_to_client = async {
        tokio::select! {
            res = io::copy(&mut out_read, &mut in_write) => res.map(|_| ())?,
            _ = ct.cancelled() => {}
        }
        in_write.shutdown().await
    };

    if let Err(err) = tokio::try_join!(client_to_server, server_to_client) {
        log::error!("{}", err);
    };

    log::debug!("closed tcp connection: {}", in_addr);
}
