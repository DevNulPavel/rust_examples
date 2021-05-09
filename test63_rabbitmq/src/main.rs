#[macro_use] mod macroses;
mod error;
mod consume_produce;
mod publish_subscribe;

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
#[allow(unused_imports)]
use self::{
    error::{
        RabbitError
    },
    consume_produce::{
        produce_consume_example
    },
    publish_subscribe::{
        pub_sub_example
    }
};

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

#[tokio::main]
async fn main() -> Result<(), RabbitError> {
    // Read env from .env file
    dotenv::dotenv().ok();

    // Friendly panic messages
    human_panic::setup_panic!();

    // Logs
    initialize_logs();

    // Examples
    // produce_consume_example()
    //     .await;
    pub_sub_example()
        .await;

    Ok(())
}
