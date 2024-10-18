mod cli_ui;
mod config;
mod error;
mod introspect;
mod local;
mod update;

use std::{
    collections::{
        HashMap
    },
    sync::{
        Arc, 
        RwLock
    },
    time::{
        Duration
    },
    env
};
use futures::{
    channel::{
        mpsc::{
            unbounded, 
            UnboundedSender
        }
    },
    SinkExt, 
    StreamExt
};
use human_panic::{
    setup_panic
};
use tokio::{
    net::{
        TcpStream
    },
    sync::{
        Mutex
    }
};
use tokio_tungstenite::{
    tungstenite::{
        Message
    },
    MaybeTlsStream, 
    WebSocketStream
};
use colored::{
    Colorize
};
use futures::{
    future::{
        Either
    }
};
use log::{
    debug, 
    error, 
    info, 
    warn
};
use tunnelto_lib::{
    *
};
pub use self::{
    cli_ui::{
        CliInterface
    },
    introspect::{
        IntrospectionAddrs
    },
    error::{
        *
    },
    config::{
        *
    }
};

pub type ActiveStreams = Arc<RwLock<HashMap<StreamId, UnboundedSender<StreamMessage>>>>;

lazy_static::lazy_static! {
    // Глобальная переменная с активными стримами
    pub static ref ACTIVE_STREAMS: ActiveStreams = Arc::new(RwLock::new(HashMap::new()));
    // Глобальная переменная с токеном повторного подключения
    pub static ref RECONNECT_TOKEN: Arc<Mutex<Option<ReconnectToken>>> = Arc::new(Mutex::new(None));
}

#[derive(Debug, Clone)]
pub enum StreamMessage {
    Data(Vec<u8>),
    Close,
}

#[tokio::main]
async fn main() {
    // Красивый вывод паники
    setup_panic!();

    // Получаем конфиг
    let mut config = match Config::get() {
        Ok(config) => config,
        Err(_) => return,
    };

    // Проверка новой версии
    update::check().await;

    // Запуск сервера, там же внутри запускается ожидание входящих данных
    let introspect_addrs = introspect::start_introspection_server(config.clone());

    loop {
        // Канал для рестарта
        let (restart_tx, mut restart_rx) = unbounded();

        // Запуск туннеля к нашим локальным серверам
        let wormhole_future = run_wormhole(config.clone(), introspect_addrs.clone(), restart_tx);

        // Ждем результата работы туннеля
        let result = futures::future::select(Box::pin(wormhole_future), restart_rx.next())
            .await;
            
        // Уже не первый запуск
        config.first_run = false;

        match result {
            Either::Left((Err(e), _)) => match e {
                Error::WebSocketError(_) | Error::NoResponseFromServer | Error::Timeout => {
                    error!("Control error: {:?}. Retrying in 5 seconds.", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
                _ => {
                    eprintln!("Error: {}", format!("{}", e).red());
                    return;
                }
            },
            Either::Right((Some(e), _)) => {
                warn!("restarting in 3 seconds...from error: {:?}", e);
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
            _ => {}
        };

        info!("restarting wormhole");
    }
}

/// Настройка туннеля к нашему сервера
async fn run_wormhole(config: Config,
                      introspect: IntrospectionAddrs,
                      mut restart_tx: UnboundedSender<Option<Error>>) -> Result<(), Error> {

    // Запускаем интерфейс
    let interface = CliInterface::start(config.clone(), introspect.clone());

    // Немного ждем
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Подключаемся к сервеному вебсокету
    let (websocket, sub_domain) = connect_to_wormhole(&config).await?;

    // Отображаем в интерфейсе успешное подключение с полученным адресом
    interface.did_connect(&sub_domain);

    // Отделяем входной и выходной потоки внешнего вебсокета
    let (mut external_ws_sender, mut external_ws_receiver) = websocket.split();

    // Канал для передачи данных 
    let (tunnel_tx, mut tunnel_rx) = unbounded::<ControlPacket>();

    // Запускаем корутину, в которой будем состоянно читать из канала и писать во внешний вебсокет
    let mut restart = restart_tx.clone();
    tokio::spawn(async move {
        loop {
            // Получаем данные из канала тунеля
            let packet = match tunnel_rx.next().await {
                Some(data) => {
                    data
                },
                // Если из канала тунеля получили пустые данные, 
                // значит нужно переподключаться
                None => {
                    warn!("control flow didn't send anything!");
                    let _ = restart.send(Some(Error::Timeout)).await;
                    return;
                }
            };

            if let Err(e) = external_ws_sender.send(Message::binary(packet.serialize())).await {
                warn!("failed to write message to tunnel websocket: {:?}", e);
                let _ = restart.send(Some(Error::WebSocketError(e))).await;
                return;
            }
        }
    });

    // Здесь будем читать из внешнего сокета и писать в канал внутренний
    loop {
        match external_ws_receiver.next().await {
            // Получили сообщение закрытия вебсокета
            Some(Ok(message)) if message.is_close() => {
                debug!("got close message");
                // Пишем в канал сообщение для рестарта
                let _ = restart_tx.send(None).await;
                return Ok(());
            }

            // Просто сообщение какое-то
            Some(Ok(message)) => {
                // Обрабатываем сообщение
                let packet = process_control_flow_message(&introspect,
                                                          tunnel_tx.clone(),
                                                          message.into_data())
                    .await
                    .map_err(|e| {
                        error!("Malformed protocol control packet: {:?}", e);
                        Error::MalformedMessageFromServer
                    })?;
                debug!("Processed packet: {:?}", packet.packet_type());
            }

            // Ошибка
            Some(Err(e)) => {
                warn!("websocket read error: {:?}", e);
                return Err(Error::Timeout);
            }

            // Данных нету
            None => {
                warn!("websocket sent none");
                return Err(Error::Timeout);
            }
        }
    }
}

/// Непосредственно подключение к серверу туннеля
async fn connect_to_wormhole(config: &Config) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, String), Error> {
    // Подключаемся к веб-сокету сервера
    let (mut websocket, _) = tokio_tungstenite::connect_async(&config.control_url).await?;

    // Отправляем сообщение "Hello"
    let client_hello = match config.secret_key.clone() {
        Some(secret_key) => {
            ClientHello::generate(config.sub_domain.clone(),
                                  ClientType::Auth { key: secret_key })
        },
        None => {
            // Если у нас есть токен переподключения - используем его
            if let Some(reconnect) = RECONNECT_TOKEN.lock().await.clone() {
                ClientHello::reconnect(reconnect)
            } else {
                ClientHello::generate(config.sub_domain.clone(), ClientType::Anonymous)
            }
        }
    };

    info!("connecting to wormhole...");

    // Кодируем сообщение в json
    let hello = serde_json::to_vec(&client_hello)
        .unwrap();
    
    // Пишем в сокет
    websocket
        .send(Message::binary(hello))
        .await
        .expect("Failed to send client hello to wormhole server.");

    // Ждем ответа
    let server_hello_data = websocket
        .next()
        .await
        .ok_or(Error::NoResponseFromServer)??
        .into_data();

    // Ответ сервера парсим
    let server_hello = serde_json::from_slice::<ServerHello>(&server_hello_data)
        .map_err(|e| {
            error!("Couldn't parse server_hello from {:?}", e);
            Error::ServerReplyInvalid
        })?;

    let sub_domain = match server_hello {
        // Успешный ответ
        ServerHello::Success{ sub_domain, client_id, .. } => {
            info!("Server accepted our connection. I am client_{}", client_id);
            sub_domain
        }
        ServerHello::AuthFailed => {
            return Err(Error::AuthenticationFailed);
        }
        ServerHello::InvalidSubDomain => {
            return Err(Error::InvalidSubDomain);
        }
        ServerHello::SubDomainInUse => {
            return Err(Error::SubDomainInUse);
        }
        ServerHello::Error(error) => {
            return Err(Error::ServerError(error));
        },
    };

    // Возвращаем серверный сокет и домен?
    Ok((websocket, sub_domain))
}

/// Обработка входного сообщения из внешнего вебсоката
async fn process_control_flow_message(introspect: &IntrospectionAddrs,
                                      mut tunnel_tx: UnboundedSender<ControlPacket>,
                                      payload: Vec<u8>) -> Result<ControlPacket, Box<dyn std::error::Error>> {
    // Парсим 
    let control_packet = ControlPacket::deserialize(&payload)?;

    match &control_packet {
        // Начало потока
        ControlPacket::Init(stream_id) => {
            info!("stream[{:?}] -> init", stream_id.to_string());
        }

        // Ping
        ControlPacket::Ping(reconnect_token) => {
            log::info!("got ping. reconnect_token={}", reconnect_token.is_some());

            // Если получили токен переподключения, тогда сохраняем его
            if let Some(reconnect) = reconnect_token {
                let _ = RECONNECT_TOKEN
                    .lock()
                    .await
                    .replace(reconnect.clone());
            }

            // А в канал передаем сообщение пинга
            let _ = tunnel_tx
                .send(ControlPacket::Ping(None))
                .await;
        }

        // Ошибка
        ControlPacket::Refused(_) => {
            return Err("unexpected control packet".into());
        },

        // Конец стрима
        ControlPacket::End(stream_id) => {
            // Айдишник стрима
            let stream_id = stream_id.clone();

            info!("got end stream [{:?}]", &stream_id);

            tokio::spawn(async move {
                // Получаем сохраненный стрим
                let stream = ACTIVE_STREAMS
                    .read()
                    .unwrap()
                    .get(&stream_id)
                    .cloned();
                
                // Если стрим валидный
                if let Some(mut tx) = stream {
                    // Ждем 5 секунд
                    tokio::time::sleep(Duration::from_secs(5)).await;

                    // Отправляем в стрим сообщение закрытия
                    let _ = tx.send(StreamMessage::Close).await.map_err(|e| {
                        error!("failed to send stream close: {:?}", e);
                    });

                    // Удаляем из списка данный стрим
                    ACTIVE_STREAMS.write().unwrap().remove(&stream_id);
                }
            });
        }

        // Полезные данные
        ControlPacket::Data(stream_id, data) => {
            info!("stream[{:?}] -> new data: {:?}",
                stream_id.to_string(),
                data.len()
            );

            // Проверяем, есть ли такой стрим?
            if !ACTIVE_STREAMS.read().unwrap().contains_key(&stream_id) {
                // Создаем стрим если его нету
                local::setup_new_stream(introspect.forward_address.port(),
                                        tunnel_tx.clone(),
                                        stream_id.clone())
                    .await;
            }

            // Находим нужный стрим
            let active_stream = ACTIVE_STREAMS.read().unwrap().get(&stream_id).cloned();

            // Прокидываем данные в него
            if let Some(mut tx) = active_stream {
                tx.send(StreamMessage::Data(data.clone())).await?;
                info!("forwarded to local tcp ({})", stream_id.to_string());
            } else {
                error!("got data but no stream to send it to.");
                let _ = tunnel_tx
                    .send(ControlPacket::Refused(stream_id.clone()))
                    .await?;
            }
        }
    };

    Ok(control_packet.clone())
}
