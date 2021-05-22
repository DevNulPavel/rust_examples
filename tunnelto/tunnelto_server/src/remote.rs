use tokio::{
    io::{ AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf },
    net::TcpStream,
};
use tracing::{ debug, error, Instrument };
use super::*;

async fn direct_to_control(mut incoming: TcpStream) {
    let mut control_socket =
        match TcpStream::connect(format!("localhost:{}", CONFIG.control_port)).await {
            Ok(s) => s,
            Err(error) => {
                tracing::warn!(?error, "failed to connect to local control server");
                return;
            }
        };

    let (mut control_r, mut control_w) = control_socket.split();
    let (mut incoming_r, mut incoming_w) = incoming.split();

    let join_1 = tokio::io::copy(&mut control_r, &mut incoming_w);
    let join_2 = tokio::io::copy(&mut incoming_r, &mut control_w);

    match futures::future::join(join_1, join_2).await {
        (Ok(_), Ok(_)) => {}
        (Err(error), _) | (_, Err(error)) => {
            tracing::error!(?error, "directing stream to control failed");
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

struct StreamWithPeekedHost {
    socket: TcpStream,
    host: String,
    forwarded_for: String,
}

/// Фильтрация входящих внешних потоков
#[tracing::instrument(skip(socket))]
async fn peek_http_request_host(mut socket: TcpStream) -> Option<StreamWithPeekedHost> {
    // Заметим, что возвращаемся если, не нашли заголовок хоста в первых 4Kb
    const MAX_HEADER_PEAK: usize = 4096;
    let mut buf = vec![0; MAX_HEADER_PEAK]; // 4kb

    tracing::debug!("checking stream headers");

    // Вычитываем данные без очистки буффера в сокете для повторного чтения
    let n = match socket.peek(&mut buf).await {
        Ok(n) => {
            n
        },
        Err(e) => {
            error!("failed to read from tcp socket to determine host: {:?}", e);
            return None;
        }
    };

    // Удостоверимся, что мы вычитали действительно данные
    if n == 0 {
        tracing::debug!("unable to peek header bytes");
        return None;
    }
    tracing::debug!("peeked {} stream bytes ", n);

    // Буффер для заголовков и зам объект запроса, буффер просто в виде контейнера
    let mut headers = [httparse::EMPTY_HEADER; 64]; // 30 seems like a generous # of headers
    let mut req = httparse::Request::new(&mut headers);

    // Парсим прилетевшие данные
    if let Err(e) = req.parse(&buf[..n]) {
        error!("failed to parse incoming http bytes: {:?}", e);
        return None;
    }

    // Обрабатываем маршрут проверки доступности
    if req.path.map(|s| s.as_bytes()) == Some(HEALTH_CHECK_PATH) {
        // Пишем в сокет ответ
        let _ = socket.write_all(HTTP_OK_RESPONSE).await.map_err(|e| {
            error!("failed to write health_check: {:?}", e);
        });

        return None;
    }

    // Ищем ип адрес для кого данный пакет
    // Адрес будет в заголовке запроса, который начинается на x: x-forwarded-for
    let headers_iter = req
        .headers
        .iter()
        .filter(|h| {
            let name = h.name.to_lowercase();
            name.as_str().eq("x-forwarded-for")
        })
        .map(|h| {
            std::str::from_utf8(h.value)
        })
        .next();
    let forwarded_for = if let Some(Ok(forwarded_for)) = headers_iter {
        forwarded_for.to_string()
    } else {
        String::default()
    };

    // Ищем адрес целевого хоста в заголовке
    let address = req
        .headers
        .iter()
        .filter(|h| {
            h
                .name
                .to_lowercase()
                .as_str()
                .eq("host")
        })
        .map(|h| {
            std::str::from_utf8(h.value)
        })
        .next();
    if let Some(Ok(host)) = address {
        // Все нашли, можно вернуть
        return Some(StreamWithPeekedHost {
            socket,
            host: host.to_string(),
            forwarded_for,
        });
    }

    tracing::info!("found no host header, dropping connection.");
    None
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Обработка нового подключения к серверу
#[tracing::instrument(skip(socket))]
pub async fn accept_connection(socket: TcpStream) {
    // Прочитаем из сокета хост, и для кого предназначено
    // Но без фактической очистки буффера в сокете
    let s = match peek_http_request_host(socket).await {
        Some(s) => {
            s
        },
        None => {
            return;
        }
    };
    let StreamWithPeekedHost{ mut socket, host, forwarded_for } = s;
    tracing::info!(%host, %forwarded_for, "new remote connection");

    // Парсим строчку хоста и ищем нашего клиента
    if CONFIG.allowed_hosts.contains(&host) {
        // Если в списке разрешенных, значит отвечаем сообщением с редиректом
        // на коренной сайт https://tunnelto.dev/
        error!("redirect to homepage");
        let _ = socket.write_all(HTTP_REDIRECT_RESPONSE).await;
        return;
    }

    // Проверяем префикс хоста
    let host = match validate_host_prefix(&host) {
        Some(sub_domain) => sub_domain,
        None => {
            error!("invalid host specified");
            let _ = socket.write_all(HTTP_INVALID_HOST_RESPONSE).await;
            return;
        }
    };

    // Если целевой хост - сервер управления - перенаправляем в него
    if host.as_str() == "wormhole" {
        direct_to_control(socket).await;
        return;
    }

    // Ищем клиент, слушающего данный хост
    let client = match Connections::find_by_host(&host) {
        Some(client) => {
            client.clone()
        },
        None => {
            // Проверяем другие экземпляры сервера, слушающие данный хост
            match network::instance_for_host(&host).await {
                Ok((instance, _)) => {
                    // Если нашли другой инстанс, перенаправляем в него
                    network::proxy_stream(instance, socket).await;
                    return;
                }
                Err(network::Error::DoesNotServeHost) => {
                    // Не нашли, пишем в ответ
                    error!(%host, "no tunnel found");
                    let _ = socket.write_all(HTTP_NOT_FOUND_RESPONSE).await;
                    return;
                }
                Err(error) => {
                    // Не нашли хост
                    error!(%host, ?error, "failed to find instance");
                    let _ = socket.write_all(HTTP_ERROR_LOCATING_HOST_RESPONSE).await;
                    return;
                }
            }
        }
    };

    // Создаем новый стрим для данного запроса
    let (active_stream, queue_rx) = ActiveStream::new(client.clone());
    let stream_id = active_stream.id.clone();
    tracing::debug!(stream_id = %active_stream.id.to_string(), 
                    "new stream connected");
                    
    let (stream, sink) = tokio::io::split(socket);

    // Сохраняем стрим
    ACTIVE_STREAMS.insert(stream_id.clone(), active_stream.clone());

    // Запускаем обработку данных из потока и запись в клиента
    let span = observability::remote_trace("process_tcp_stream");
    tokio::spawn(
        async move {
            process_tcp_stream(active_stream, stream).await;
        }
        .instrument(span),
    );

    // Читаем клиента и пишем в сокет
    let span = observability::remote_trace("tunnel_to_stream");
    tokio::spawn(
        async move {
            tunnel_to_stream(host, stream_id, sink, queue_rx).await;
        }
        .instrument(span),
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn validate_host_prefix(host: &str) -> Option<String> {
    let url = format!("http://{}", host);

    let host = url::Url::parse(&url)
        .ok()
        .and_then(|u| {
            u
                .host()
                .map(|h| {
                    h.to_owned()
                })
        });

    let host = match host
    {
        Some(domain) => {
            domain.to_string()
        },
        None => {
            error!("invalid host header");
            return None;
        }
    };

    let domain_segments = host.split(".").collect::<Vec<&str>>();
    let prefix = &domain_segments[0];
    let remaining = &domain_segments[1..].join(".");

    if CONFIG.allowed_hosts.contains(remaining) {
        Some(prefix.to_string())
    } else {
        None
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Response Constants
const HTTP_REDIRECT_RESPONSE:&'static [u8] = b"HTTP/1.1 301 Moved Permanently\r\nLocation: https://tunnelto.dev/\r\nContent-Length: 20\r\n\r\nhttps://tunnelto.dev";
const HTTP_INVALID_HOST_RESPONSE: &'static [u8] =
    b"HTTP/1.1 400\r\nContent-Length: 23\r\n\r\nError: Invalid Hostname";
const HTTP_NOT_FOUND_RESPONSE: &'static [u8] =
    b"HTTP/1.1 404\r\nContent-Length: 23\r\n\r\nError: Tunnel Not Found";
const HTTP_ERROR_LOCATING_HOST_RESPONSE: &'static [u8] =
    b"HTTP/1.1 500\r\nContent-Length: 27\r\n\r\nError: Error finding tunnel";
const HTTP_TUNNEL_REFUSED_RESPONSE: &'static [u8] =
    b"HTTP/1.1 500\r\nContent-Length: 32\r\n\r\nTunnel says: connection refused.";
const HTTP_OK_RESPONSE: &'static [u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
const HEALTH_CHECK_PATH: &'static [u8] = b"/0xDEADBEEF_HEALTH_CHECK";

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Process Messages from the control path in & out of the remote stream
#[tracing::instrument(skip(tunnel_stream, tcp_stream))]
async fn process_tcp_stream(mut tunnel_stream: ActiveStream, mut tcp_stream: ReadHalf<TcpStream>) {
    // send initial control stream init to client
    control_server::send_client_stream_init(tunnel_stream.clone()).await;

    // now read from stream and forward to clients
    let mut buf = [0; 1024];

    loop {
        // client is no longer connected
        if Connections::get(&tunnel_stream.client.id).is_none() {
            debug!("client disconnected, closing stream");
            let _ = tunnel_stream.tx.send(StreamMessage::NoClientTunnel).await;
            tunnel_stream.tx.close_channel();
            return;
        }

        // read from stream
        let n = match tcp_stream.read(&mut buf).await {
            Ok(n) => n,
            Err(e) => {
                error!("failed to read from tcp socket: {:?}", e);
                return;
            }
        };

        if n == 0 {
            debug!("stream ended");
            let _ = tunnel_stream
                .client
                .tx
                .send(ControlPacket::End(tunnel_stream.id.clone()))
                .await
                .map_err(|e| {
                    error!("failed to send end signal: {:?}", e);
                });
            return;
        }

        debug!("read {} bytes", n);

        let data = &buf[..n];
        let packet = ControlPacket::Data(tunnel_stream.id.clone(), data.to_vec());

        match tunnel_stream.client.tx.send(packet.clone()).await {
            Ok(_) => debug!(client_id = %tunnel_stream.client.id, "sent data packet to client"),
            Err(_) => {
                error!("failed to forward tcp packets to disconnected client. dropping client.");
                Connections::remove(&tunnel_stream.client);
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[tracing::instrument(skip(sink, stream_id, queue))]
async fn tunnel_to_stream(
    subdomain: String,
    stream_id: StreamId,
    mut sink: WriteHalf<TcpStream>,
    mut queue: UnboundedReceiver<StreamMessage>,
) {
    loop {
        let result = queue.next().await;

        let result = if let Some(message) = result {
            match message {
                StreamMessage::Data(data) => Some(data),
                StreamMessage::TunnelRefused => {
                    tracing::debug!(?stream_id, "tunnel refused");
                    let _ = sink.write_all(HTTP_TUNNEL_REFUSED_RESPONSE).await;
                    None
                }
                StreamMessage::NoClientTunnel => {
                    tracing::info!(%subdomain, ?stream_id, "client tunnel not found");
                    let _ = sink.write_all(HTTP_NOT_FOUND_RESPONSE).await;
                    None
                }
            }
        } else {
            None
        };

        let data = match result {
            Some(data) => data,
            None => {
                tracing::debug!("done tunneling to sink");
                let _ = sink.shutdown().await.map_err(|_e| {
                    error!("error shutting down tcp stream");
                });

                ACTIVE_STREAMS.remove(&stream_id);
                return;
            }
        };

        let result = sink.write_all(&data).await;

        if let Some(error) = result.err() {
            tracing::warn!(?error, "stream closed, disconnecting");
            return;
        }
    }
}
