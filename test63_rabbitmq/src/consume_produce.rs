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
use tracing::{
    info,
    error,
    info_span,
    instrument
};
use tracing_futures::{
    Instrument
};
use crate::{
    ok_or,
    unwrap_or_else
};

#[instrument(skip(consumer))]
async fn consume_data(mut consumer: Consumer) {
    while let Some(data) = consumer.next().await{
        let (_channel, delivery) = unwrap_or_else!(data, err => {
            error!(%err, "Rabbit consume error");
            continue;
        });

        let text = unwrap_or_else!(std::str::from_utf8(delivery.data.as_ref()), err => {
            error!(?err, "UTF-8 parse failed");
            continue;
        });

        info!(?text, "Rabbit data received");

        tokio::time::sleep(std::time::Duration::from_millis(1000))
            .await;

        ok_or!(delivery.ack(BasicAckOptions::default()).await, err => {
            error!(%err, "Ack delivery failed");
        });
    }
}

#[instrument(skip(channel))]
async fn produce_data(channel: Channel, queue: Queue) {
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(450))
            .await;

        let pub_res = channel.basic_publish("", 
                                            queue.name().as_str(), 
                                                BasicPublishOptions{
                                                ..Default::default()
                                            }, 
                                            b"My message data".to_vec(), 
                                            BasicProperties::default())
            .await;
        
        ok_or!(pub_res, err => {
            error!(%err, "Data produce failed");
        });
    }
}

#[instrument]
pub async fn produce_consume_example() {

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
    tokio::join!(produce_data(ch1, queue), 
                 consume_data(consumer.clone())
                     .instrument(info_span!("consumer_1")), 
                 consume_data(consumer.clone())
                     .instrument(info_span!("consumer_2")),
                 consume_data(consumer)
                     .instrument(info_span!("consumer_3")));
    info!("Consumers and producers complete");
}