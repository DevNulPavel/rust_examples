mod error;

use tap::{
    TapFallible
};
use tracing::{
    info,
    error,
    info_span,
    instrument
};
use tracing_futures::{
    Instrument
};
use tracing_subscriber::{
    prelude::{
        *
    },
    fmt::{
        format::{
            FmtSpan
        }
    }
};
use zeromq::{
    prelude::{
        *
    },
    BlockingSend,
    BlockingRecv,
    NonBlockingSend,
    NonBlockingRecv,
    MultiPeerBackend,
    SocketBackend,
    Socket,
    ReqSocket,
    RepSocket,
    PushSocket,
    PullSocket,
    DealerSocket,
    ZmqMessage
};
use futures::{
    StreamExt
};
use self::{
    error::{
        AppError
    }
};

///////////////////////////////////////////////////////////////////////////////////////////

fn initialize_logs() {
    // Логи в stdout
    let stdoud_sub = tracing_subscriber::fmt::layer()
        .pretty()
        .with_writer(std::io::stdout)
        .with_span_events(FmtSpan::NONE);

    // Суммарный обработчик
    let full_subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env()
                .and_then(stdoud_sub));

    // Установка по-умолчанию
    tracing::subscriber::set_global_default(full_subscriber).unwrap();    
}

macro_rules! unwrap_or {
    ($r:expr, $err_val:ident => $or:expr ) => {
        match $r {
            Ok(data) => {
                data
            },
            Err($err_val) => {
                $or
            }
        }
    };
}

macro_rules! ok_or {
    ($r:expr, $err_val:ident => $or:expr ) => {
        if let Err($err_val) = $r {
            $or
        }
    };
}

#[instrument]
async fn consume_data(addr: &str){
    // Забиндиться мы можем лишь на один единстенный адрес??
    let mut receive_socket = RepSocket::new();
    receive_socket
        .bind(addr)
        .await
        .tap_err(|err|{
            error!(%err, "Pull socket connect failed");
        })
        .unwrap();  

    loop {
        let received_data = unwrap_or!(receive_socket.recv().await, err => {
            error!(%err, "Data receive");
            continue;
        });

        let text = unwrap_or!(std::str::from_utf8(received_data.data.as_ref()), err => {
            error!(%err, "Data parse failed");
            continue;
        });

        // info!(%text, "Received");

        let response = format!("From {}: {}", addr, text.to_uppercase());

        tokio::time::sleep(std::time::Duration::from_millis(200))
            .await;

        ok_or!(receive_socket.send(response.into()).await, err => {
            error!(%err, "Response send failed");
            continue;
        });
    }
}

#[instrument(skip(addresses))]
async fn request_sender(addresses: &[&str]) {
    assert!(addresses.len() > 0, "Consumers coount must be greater than 1");

    // Создаем сокет запроса
    let mut send_socket = ReqSocket::new();

    // Подключаться мы можем к любому количеству клиентов
    // запросы будут обрабатываться по алгоритму round-robin
    for addr in addresses.iter() {
        send_socket
            .connect(addr)
            .await
            .tap_err(|err|{
                error!(%err, "Request socket connect failed");
            })
            .unwrap();
    }

    loop {
        tokio::time::sleep(std::time::Duration::from_millis(200))
            .await;

        ok_or!(send_socket.send("Test data".into()).await, err => {
            error!(%err, "Data send");
            continue;
        });

        let result: ZmqMessage = unwrap_or!(send_socket.recv().await, err => {
            error!(%err, "Response receive");
            continue;
        });

        info!(data = ?result.data, "Response")
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Read env from .env file
    dotenv::dotenv().ok();

    // Friendly panic messages
    human_panic::setup_panic!();

    // Logs
    initialize_logs();

    // Producers and consumers
    let addresses = [
        "tcp://127.0.0.1:5551",
        "tcp://127.0.0.1:5552",
        "tcp://127.0.0.1:5553",
    ];
    tokio::join!(consume_data(addresses[0])
                    .instrument(info_span!("consumer_1")),
                 consume_data(addresses[1])
                    .instrument(info_span!("consumer_2")),
                 consume_data(addresses[2])
                    .instrument(info_span!("consumer_3")),
                 request_sender(&addresses)
                    .instrument(info_span!("sender_1")));
    info!("Consumers and producers finished");

    Ok(())
}
