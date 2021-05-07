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
    BlockingSend,
    BlockingRecv,
    Socket,
    ReqSocket,
    RepSocket,
    ZmqMessage
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
async fn consume_data(){
    // Reply server socket
    let mut rep_socket = RepSocket::new();
    rep_socket
        .connect("tcp://0.0.0.0:5555")
        .await
        .tap_err(|err|{
            error!(%err, "Reply socket connect failed");
        })
        .unwrap();

    loop {
        let received_data = unwrap_or!(rep_socket.recv().await, err => {
            error!(%err, "Data receive");
            continue;
        });

        let text = unwrap_or!(std::str::from_utf8(received_data.data.as_ref()), err => {
            error!(%err, "Data parse failed");
            continue;
        });

        let response = text.to_uppercase();
        tokio::time::sleep(std::time::Duration::from_millis(500))
            .await;

        ok_or!(rep_socket.send(response.into()).await, err => {
            error!(%err, "Response send failed");
            continue;
        });
    }
}

#[instrument(err)]
async fn request_sender() -> Result<(), AppError>{
    // Creates request socket
    let mut req_socket = ReqSocket::new();
    req_socket
        .connect("tcp://127.0.0.1:5555")
        .await
        .tap_err(|err|{
            error!(%err, "Request socket connect failed");
        })?;

    loop {
        ok_or!(req_socket.send("Test data".into()).await, err => {
            error!(%err, "Data send");
            continue;
        });

        let result: ZmqMessage = unwrap_or!(req_socket.recv().await, err => {
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
    let join_res = tokio::join!(consume_data()
                                    .instrument(info_span!("consumer_1")), 
                                consume_data()
                                    .instrument(info_span!("consumer_2")),
                                consume_data()
                                    .instrument(info_span!("consumer_3")),
                                request_sender());
    info!("Consumers and producers finished");

    // Check results
    join_res.3
        .tap_err(|err|{
            error!(%err, "Sender error")
        })?;

    Ok(())
}
