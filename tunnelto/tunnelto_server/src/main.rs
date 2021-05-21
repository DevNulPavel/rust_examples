mod connected_clients;
mod active_stream;
mod auth;
mod control_server;
mod remote;
mod config;
mod network;
mod observability;

use std::{
    sync::{
        Arc
    }
};
use futures::{
    channel::{
        mpsc::{
            unbounded, 
            UnboundedReceiver, 
            UnboundedSender
        }
    },
    stream::{
        SplitSink, 
        SplitStream
    },
    SinkExt, 
    StreamExt
};
use warp::{
    ws::{
        Message, 
        WebSocket, 
        Ws
    },
    Filter
};
use tracing::{
    level_filters::{
        LevelFilter
    },
    error, 
    info, 
    Instrument
};
use tracing_subscriber::{
    layer::{
        SubscriberExt
    },
    registry
};
use tracing_honeycomb::{
    libhoney
};
use dashmap::{
    DashMap
};
use tokio::{
    net::{
        TcpListener
    }
};
use lazy_static::{
    lazy_static
};
use tunnelto_lib::{
    *
};
use self::{
    connected_clients::{
        *
    },
    active_stream::{
        *
    },
    config::{
        Config
    },
    auth::{
        auth_db,
        client_auth
    },
    auth_db::{
        AuthDbService
    }
};

//////////////////////////////////////////////////////////////////////////////////////////

lazy_static! {
    pub static ref CONNECTIONS: Connections = Connections::new();
    pub static ref ACTIVE_STREAMS: ActiveStreams = Arc::new(DashMap::new());
    pub static ref AUTH_DB_SERVICE: AuthDbService = AuthDbService::new().expect("failed to init auth-service");
    pub static ref CONFIG: Config = Config::from_env();

    // To disable all authentication:
    // pub static ref AUTH_DB_SERVICE: crate::auth::NoAuth = crate::auth::NoAuth;
}

#[tokio::main]
async fn main() {
    // Есть ли ключ для отслеживания телеметрии?
    if let Some(api_key) = CONFIG.honeycomb_api_key.clone() {
        info!("configuring observability layer");

        // Конфиг для слоя tracing
        let honeycomb_config = libhoney::Config {
            options: libhoney::client::Options {
                api_key,
                dataset: "t2-service".to_string(),
                ..libhoney::client::Options::default()
            },
            transmission_options: libhoney::transmission::Options {
                max_batch_size: 50,
                max_concurrent_batches: 10,
                batch_timeout: std::time::Duration::from_millis(1000),
                pending_work_capacity: 5_000,
                user_agent_addition: None,
            },
        };
        let telemetry_layer = tracing_honeycomb::new_honeycomb_telemetry_layer("t2-service", honeycomb_config);

        // Создаем subscriber, вывод в телеметрию + stdout
        let subscriber = registry::Registry::default()
            .with(LevelFilter::INFO)
            .with(tracing_subscriber::fmt::Layer::default())
            .with(telemetry_layer);

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting global default failed");
    } else {
        // Либо вывод делаем только в stdout
        let subscriber = registry::Registry::default()
            .with(LevelFilter::INFO)
            .with(tracing_subscriber::fmt::Layer::default());
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting global default failed");
    };

    tracing::info!("starting server!");

    // Запуск сервера управления
    control_server::spawn(([0, 0, 0, 0], CONFIG.control_port));
    info!("started tunnelto server on 0.0.0.0:{}", CONFIG.control_port);

    // Запуск внутренней сети между серверами
    network::spawn(([0, 0, 0, 0, 0, 0, 0, 0], CONFIG.internal_network_port));
    info!("start network service on [::]:{}", CONFIG.internal_network_port);

    // Создаем слушающий сокет
    let listen_addr = format!("[::]:{}", CONFIG.remote_port);
    let listener = TcpListener::bind(&listen_addr)
        .await
        .expect("failed to bind");
    info!("listening on: {}", &listen_addr);        

    loop {
        // Прилетело подключение
        let socket = match listener.accept().await {
            Ok((socket, _)) => {
                socket
            },
            _ => {
                error!("failed to accept socket");
                continue;
            }
        };

        // Запускаем обработку данных
        tokio::spawn({
            let fut = async move {
                remote::accept_connection(socket).await;
            };
            fut.instrument(observability::remote_trace("remote_connect"))
        });
    }
}
