use crate::shared::args::Args;
use iggy::client::Client;
use iggy::compression::compression_algorithm::CompressionAlgorithm;
use iggy::consumer::{Consumer, ConsumerKind};
use iggy::error::IggyError;
use iggy::identifier::Identifier;
use iggy::messages::poll_messages::PollingStrategy;
use iggy::models::messages::PolledMessage;
use iggy::utils::expiry::IggyExpiry;
use iggy::utils::topic_size::MaxTopicSize;
use tracing::info;

type MessageHandler = dyn Fn(&PolledMessage) -> Result<(), Box<dyn std::error::Error>>;

pub async fn init_by_consumer(args: &Args, client: &dyn Client) {
    let (stream_id, topic_id, partition_id) = (
        args.stream_id.clone(),
        args.topic_id.clone(),
        args.partition_id,
    );
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    let stream_id = stream_id.try_into().unwrap();
    let topic_id = topic_id.try_into().unwrap();
    loop {
        interval.tick().await;
        info!("Validating if stream: {stream_id} exists..");
        let stream = client.get_stream(&stream_id).await;
        if stream.is_err() {
            continue;
        }

        let stream = stream.unwrap();
        if stream.is_none() {
            continue;
        }

        info!("Stream: {stream_id} was found.");
        break;
    }
    loop {
        interval.tick().await;
        info!("Validating if topic: {} exists..", topic_id);
        let topic = client.get_topic(&stream_id, &topic_id).await;
        if topic.is_err() {
            continue;
        }

        let topic = topic.unwrap();
        if topic.is_none() {
            continue;
        }

        let topic = topic.unwrap();
        info!("Topic: {} was found.", topic_id);
        if topic.partitions_count >= partition_id {
            break;
        }

        panic!(
            "Topic: {} has only {} partition(s), but partition: {} was requested.",
            topic_id, topic.partitions_count, partition_id
        );
    }
}

pub async fn init_by_producer(args: &Args, client: &dyn Client) -> Result<(), IggyError> {
    let stream_id = args.stream_id.clone().try_into()?;
    let topic_name = args.topic_id.clone();
    let stream = client.get_stream(&stream_id).await?;
    if stream.is_some() {
        return Ok(());
    }

    info!("Stream does not exist, creating...");
    client.create_stream(&args.stream_id, None).await?;
    client
        .create_topic(
            &stream_id,
            &topic_name,
            args.partitions_count,
            CompressionAlgorithm::from_code(args.compression_algorithm)?,
            None,
            None,
            IggyExpiry::NeverExpire,
            MaxTopicSize::ServerDefault,
        )
        .await?;
    Ok(())
}

pub async fn consume_messages(
    args: &Args,
    client: &dyn Client,
    handle_message: &MessageHandler,
) -> Result<(), Box<dyn std::error::Error>> {
    let interval = args.get_interval();
    info!("Messages will be polled by consumer: {} from stream: {}, topic: {}, partition: {} with interval {}.",
        args.consumer_id, args.stream_id, args.topic_id, args.partition_id, interval.map_or("none".to_string(), |i| i.as_human_time_string()));

    let stream_id = args.stream_id.clone().try_into()?;
    let topic_id = args.topic_id.clone().try_into()?;
    let mut interval = interval.map(|interval| tokio::time::interval(interval.get_duration()));
    let mut consumed_batches = 0;
    let consumer = Consumer {
        kind: ConsumerKind::from_code(args.consumer_kind)?,
        id: Identifier::numeric(args.consumer_id).unwrap(),
    };

    loop {
        if args.message_batches_limit > 0 && consumed_batches == args.message_batches_limit {
            info!("Consumed {consumed_batches} batches of messages, exiting.");
            return Ok(());
        }

        if let Some(interval) = &mut interval {
            interval.tick().await;
        }

        let polled_messages = client
            .poll_messages(
                &stream_id,
                &topic_id,
                Some(args.partition_id),
                &consumer,
                &PollingStrategy::next(),
                args.messages_per_batch,
                true,
            )
            .await?;
        if polled_messages.messages.is_empty() {
            info!("No messages found.");
            continue;
        }
        consumed_batches += 1;
        for message in polled_messages.messages {
            handle_message(&message)?;
        }
    }
}
