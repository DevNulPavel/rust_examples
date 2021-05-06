mod error;

use lapin::{
    options::{
        QueueDeclareOptions,
        BasicConsumeOptions,
        BasicAckOptions,
        BasicPublishOptions
    },
    types::{
        FieldTable
    },
    BasicProperties,
    Consumer,
    Queue,
    Channel,
    Connection, 
    ConnectionProperties,
};
use tokio_amqp::{
    LapinTokioExt
};
use futures::{
    StreamExt
};
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
use self::{
    error::{
        RabbitError
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

#[instrument(err, skip(consumer))]
async fn consume_data(mut consumer: Consumer) -> Result<(), RabbitError>{
    while let Some(data) = consumer.next().await{
        match data {
            // 
            Ok((_channel, delivery)) => {
                match std::str::from_utf8(delivery.data.as_ref()){
                    Ok(text) => {
                        info!(?text, "Rabbit data received");
                    },
                    Err(err) => {
                        error!(?err, "UTF-8 parse failed");
                    }
                }

                tokio::time::sleep(std::time::Duration::from_millis(1000))
                    .await;

                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .tap_err(|err|{
                        error!(%err, "Ack delivery failed");
                    })?;
            },
            Err(err) => {
                error!(%err, "Rabbit consume error")
            }
        }
    }
    Ok(())
}

#[instrument(err, skip(channel))]
async fn produce_data(channel: Channel, queue: Queue) -> Result<(), RabbitError>{
    loop {
        channel
            .basic_publish("", 
                        queue.name().as_str(), 
                        BasicPublishOptions{
                            ..Default::default()
                        }, 
                        b"My message data".to_vec(), 
                        BasicProperties::default())
            .await
            .tap_err(|err|{
                error!(%err, "Data produce failed");
            })?;
        
        tokio::time::sleep(std::time::Duration::from_millis(450))
            .await;
    }
}

#[tokio::main]
async fn main() -> Result<(), RabbitError> {
    // Read env from .env file
    dotenv::dotenv().ok();

    // Friendly panic messages
    human_panic::setup_panic!();

    // Logs
    initialize_logs();

    // Start rabbit connection
    let rabbit_connection_properties = ConnectionProperties::default()
        .with_tokio();
    let rabbit_conn = Connection::connect("amqp://127.0.0.1:5672/%2f", rabbit_connection_properties)
        .await
        .expect("Rabbit connection failed");
    info!("Rabbit connection established");

    // Create anonimous channels
    let ch1 = rabbit_conn.create_channel()
        .await
        .expect("Channel create failed");
    let ch2 = rabbit_conn.create_channel()
        .await
        .expect("Channel create failed");
    info!("Channels created");

    // Queue
    let queue = ch1
        .queue_declare("test_queue", 
                        QueueDeclareOptions{
                            exclusive: true, // Только один получатель
                            ..Default::default()
                        }, 
                        FieldTable::default())
        .await
        .expect("Queue create failed");
    info!("Queue created");

    // Consumer
    let consumer = ch2
        .basic_consume("test_queue", 
                       "consumer_tag", 
                       BasicConsumeOptions{
                           ..Default::default()
                       },
                       FieldTable::default())
        .await
        .expect("Consumer create failed");
    info!("Consumer created");

    // Producers and consumers
    let join_res = tokio::join!(produce_data(ch1, queue), 
                                consume_data(consumer.clone())
                                    .instrument(info_span!("consumer_1")), 
                                consume_data(consumer.clone())
                                    .instrument(info_span!("consumer_2")),
                                consume_data(consumer)
                                    .instrument(info_span!("consumer_3")));
    info!("Consumers and producers created");

    // Check results
    join_res.0?;
    join_res.1?;
    join_res.2?;
    join_res.3?;

    Ok(())
}
