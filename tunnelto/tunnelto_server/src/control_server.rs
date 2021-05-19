use chrono::{
    Utc
};
use std::{
    net::{
        IpAddr, 
        SocketAddr
    },
    str::{
        FromStr
    },
    time::{
        Duration
    }
};
use tracing::{
    error, 
    Instrument
};
use warp::{
    Rejection
};
use crate::{
    auth::{
        reconnect_token::{ 
            ReconnectTokenPayload
        },
        client_auth::{
            ClientHandshake
        }
    }
};
use super::{
    *
};

pub fn spawn<A: Into<SocketAddr>>(addr: A) {
    // Маршрут проверки доступности
    let health_check = warp::get()
        .and(warp::path("health_check"))
        .map(|| {
            tracing::debug!("Health Check #2 triggered");
            "ok"
        });

    // Маршрут подключения к туннель-серверу
    let client_conn = warp::path("wormhole")
        .and(client_ip())
        .and(warp::ws())
        .map(move |client_ip: IpAddr, ws: Ws| {
            ws.on_upgrade(move |w| {
                // После апгрейда начинаем обработку соединения
                let fut = async move {
                    handle_new_connection(client_ip, w)
                        .await 
                };
                fut.instrument(observability::remote_trace("handle_websocket"))
            })
        });

    // Суммарный маршрут
    let routes = client_conn
        .or(health_check);

    // Запускаем сервер
    tokio::spawn(warp::serve(routes)
                    .run(addr.into()));
}

/// Создание warp фильтра для извлечения ip
fn client_ip() -> impl Filter<Extract = (IpAddr,), Error = Rejection> + Copy {
    warp::any()
        .and(warp::header::optional("Fly-Client-IP"))
        .and(warp::header::optional("X-Forwarded-For"))
        .and(warp::addr::remote())
        .map(|client_ip: Option<String>, fwd: Option<String>, remote: Option<SocketAddr>| {
            let client_ip = client_ip
                .map(|s| IpAddr::from_str(&s).ok())
                .flatten();
            let fwd = fwd
                .map(|s| {
                    s.split(",")
                        .into_iter()
                        .next()
                        .map(IpAddr::from_str)
                        .map(Result::ok)
                        .flatten()
                })
                .flatten();
            let remote = remote.map(|r| r.ip());
            client_ip
                .or(fwd)
                .or(remote)
                .unwrap_or(IpAddr::from([0, 0, 0, 0]))
        })
}

// Обработка нового websocket подключения
#[tracing::instrument(skip(websocket))]
async fn handle_new_connection(client_ip: IpAddr, websocket: WebSocket) {
    // Проверим, не является ли данный IP адрес заблокированным
    if CONFIG.blocked_ips.contains(&client_ip) {
        tracing::warn!(?client_ip, "client ip is on block list, denying connection");
        
        // Закрываем подключение
        let _ = websocket
            .close()
            .await;
        return;
    }

    // Выполняем рукопожатие для получения домена и отправки клиенту
    let (websocket, handshake) = match try_client_handshake(websocket).await {
        Some(ws) => ws,
        None => return,
    };

    // Подключились
    tracing::info!(client_ip = %client_ip, subdomain = %handshake.sub_domain, "open tunnel");

    // Создаем объект клиента и сохраняем его в общем контейнере
    let (tx, rx) = unbounded::<ControlPacket>();
    let mut client = ConnectedClient {
        id: handshake.id,
        host: handshake.sub_domain,
        is_anonymous: handshake.is_anonymous,
        tx,
    };
    Connections::add(client.clone());

    // Разделяем вебсокет на читатель и писатель
    let (sink, stream) = websocket.split();


    // Запускаем корутину записи данных в вебсокет
    tokio::spawn({
        let client_clone = client.clone();
        let fut = async move {
            tunnel_client(client_clone, sink, rx).await;
        };
        fut.instrument(observability::remote_trace("tunnel_client"))
    });

    // Запускаем корутину чтения данных из вебсокета
    tokio::spawn({
        let client_clone = client.clone();
        let fut = async move {
            process_client_messages(client_clone, stream).await;
        };
        fut.instrument(observability::remote_trace("process_client"))
    });

    // Отдельная корутина по отправке PING/PONG
    tokio::spawn({
        let fut = async move {
            loop {
                tracing::trace!("sending ping");

                // create a new reconnect token for anonymous clients
                let reconnect_token = if client.is_anonymous {
                    let payload = ReconnectTokenPayload {
                        sub_domain: client.host.clone(),
                        client_id: client.id.clone(),
                        expires: Utc::now() + chrono::Duration::minutes(2),
                    };
                    payload
                        .into_token(&CONFIG.master_sig_key)
                        .map_err(|e| error!("unable to create reconnect token: {:?}", e))
                        .ok()
                } else {
                    None
                };

                match client.tx.send(ControlPacket::Ping(reconnect_token)).await {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::debug!("Failed to send ping: {:?}, removing client", e);

                        // Если мы не смогли отправить сообщение, значит клиент отвалился - надо уго удалить
                        Connections::remove(&client);
                        return;
                    }
                };

                tokio::time::sleep(Duration::new(PING_INTERVAL, 0)).await;
            }
        };
        fut.instrument(observability::remote_trace("control_ping"))
    });
}

#[tracing::instrument(skip(websocket))]
async fn try_client_handshake(websocket: WebSocket) -> Option<(WebSocket, ClientHandshake)> {
    // Аутентификация вебсокета для получения домена
    let (mut websocket, client_handshake) = client_auth::auth_client_handshake(websocket).await?;

    // Send server hello success
    let data = serde_json::to_vec(&ServerHello::Success {
                                        sub_domain: client_handshake.sub_domain.clone(),
                                        client_id: client_handshake.id.clone()
                                    })
        .unwrap_or_default();

    // Отправляем клиенту успешную аутентификацию
    let send_result = websocket
        .send(Message::binary(data))
        .await;

    if let Err(error) = send_result {
        error!(?error, "aborting...failed to write server hello");
        return None;
    }

    tracing::debug!("new client connected: {:?}{}",
        &client_handshake.id,
        if client_handshake.is_anonymous {
            " (anonymous)"
        } else {
            ""
        }
    );
    Some((websocket, client_handshake))
}

/// Send the client a "stream init" message
pub async fn send_client_stream_init(mut stream: ActiveStream) {
    match stream
        .client
        .tx
        .send(ControlPacket::Init(stream.id.clone()))
        .await
    {
        Ok(_) => {
            tracing::debug!("sent control to client: {}", &stream.client.id);
        }
        Err(_) => {
            tracing::debug!("removing disconnected client: {}", &stream.client.id);
            Connections::remove(&stream.client);
        }
    }
}

/// Обрабатываем входящие сообщения из вебсокета
#[tracing::instrument(skip(client_conn))]
async fn process_client_messages(client: ConnectedClient, mut client_conn: SplitStream<WebSocket>) {
    loop {
        let result = client_conn.next().await;

        let message = match result {
            // handle protocol message
            Some(Ok(msg)) if (msg.is_binary() || msg.is_text()) && !msg.as_bytes().is_empty() => {
                msg.into_bytes()
            }
            // handle close with reason
            Some(Ok(msg)) if msg.is_close() && !msg.as_bytes().is_empty() => {
                tracing::debug!(close_reason=?msg, "got close");

                // Удаляем клиента если от отключился
                Connections::remove(&client);
                return;
            }
            _ => {
                tracing::debug!(?client.id, "goodbye client");

                // Удаляем клиента если от отключился
                Connections::remove(&client);
                return;
            }
        };

        let packet = match ControlPacket::deserialize(&message) {
            Ok(packet) => packet,
            Err(error) => {
                error!(?error, "invalid data packet");
                continue;
            }
        };

        // Обрабатываем данные
        let (stream_id, message) = match packet {
            ControlPacket::Data(stream_id, data) => {
                tracing::debug!(?stream_id, num_bytes=?data.len(),"forwarding to stream");
                (stream_id, StreamMessage::Data(data))
            }
            ControlPacket::Refused(stream_id) => {
                tracing::debug!("tunnel says: refused");
                (stream_id, StreamMessage::TunnelRefused)
            }
            ControlPacket::Init(_) | ControlPacket::End(_) => {
                error!("invalid protocol control::init message");
                continue;
            }
            ControlPacket::Ping(_) => {
                tracing::trace!("pong");
                Connections::add(client.clone());
                continue;
            }
        };

        let stream = ACTIVE_STREAMS
            .get(&stream_id)
            .map(|s| s.value().clone());

        if let Some(mut stream) = stream {
            // Пишем данные в канал
            let _ = stream
                .tx
                .send(message)
                .await
                .map_err(|error| {
                    tracing::trace!(?error, "Failed to send to stream tx");
                });
        }
    }
}

/// Отправка данных в вебсокет из канала
#[tracing::instrument(skip(sink, queue))]
async fn tunnel_client(client: ConnectedClient,
                       mut sink: SplitSink<WebSocket, Message>,
                       mut queue: UnboundedReceiver<ControlPacket>) {
    loop {
        match queue.next().await {
            Some(packet) => {
                let result = sink.send(Message::binary(packet.serialize())).await;
                if let Err(error) = result {
                    tracing::trace!(?error, "client disconnected: aborting.");

                    // Если не отправили, удаляем клиента
                    Connections::remove(&client);
                    return;
                }
            }
            None => {
                tracing::debug!("ending client tunnel");
                return;
            }
        };
    }
}
