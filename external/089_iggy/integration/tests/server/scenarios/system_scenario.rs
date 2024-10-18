use bytes::Bytes;
use std::str::FromStr;

use crate::server::scenarios::{
    get_consumer_group, leave_consumer_group, CONSUMER_GROUP_ID, CONSUMER_GROUP_NAME, CONSUMER_ID,
    CONSUMER_KIND, MESSAGES_COUNT, PARTITIONS_COUNT, PARTITION_ID, STREAM_ID, STREAM_NAME,
    TOPIC_ID, TOPIC_NAME,
};
use iggy::client::{
    ConsumerGroupClient, ConsumerOffsetClient, MessageClient, PartitionClient, StreamClient,
    SystemClient, TopicClient, UserClient,
};
use iggy::clients::client::IggyClient;
use iggy::compression::compression_algorithm::CompressionAlgorithm;
use iggy::consumer::Consumer;
use iggy::error::IggyError;
use iggy::identifier::Identifier;
use iggy::messages::poll_messages::PollingStrategy;
use iggy::messages::send_messages::{Message, Partitioning};
use iggy::models::messages::PolledMessage;
use iggy::users::defaults::{DEFAULT_ROOT_PASSWORD, DEFAULT_ROOT_USERNAME};
use iggy::utils::byte_size::IggyByteSize;
use iggy::utils::expiry::IggyExpiry;
use iggy::utils::topic_size::MaxTopicSize;
use integration::test_server::{assert_clean_system, ClientFactory};

pub async fn run(client_factory: &dyn ClientFactory) {
    let client = client_factory.create_client().await;
    let client = IggyClient::create(client, None, None);

    let consumer = Consumer {
        kind: CONSUMER_KIND,
        id: Identifier::numeric(CONSUMER_ID).unwrap(),
    };

    // 0. Ping server
    client.ping().await.unwrap();

    // 1. Login as root user
    client
        .login_user(DEFAULT_ROOT_USERNAME, DEFAULT_ROOT_PASSWORD)
        .await
        .unwrap();

    // 2. Ensure that streams do not exist
    let streams = client.get_streams().await.unwrap();
    assert!(streams.is_empty());

    // 3. Create the stream
    let stream = client
        .create_stream(STREAM_NAME, Some(STREAM_ID))
        .await
        .unwrap();

    assert_eq!(stream.id, STREAM_ID);
    assert_eq!(stream.name, STREAM_NAME);

    // 4. Get streams and validate that created stream exists
    let streams = client.get_streams().await.unwrap();
    assert_eq!(streams.len(), 1);
    let stream = streams.first().unwrap();
    assert_eq!(stream.id, STREAM_ID);
    assert_eq!(stream.name, STREAM_NAME);
    assert_eq!(stream.topics_count, 0);
    assert_eq!(stream.size, 0);
    assert_eq!(stream.messages_count, 0);

    // 5. Get stream details by ID
    let stream = client
        .get_stream(&Identifier::numeric(STREAM_ID).unwrap())
        .await
        .unwrap()
        .expect("Failed to get stream");

    assert_eq!(stream.id, STREAM_ID);
    assert_eq!(stream.name, STREAM_NAME);
    assert_eq!(stream.topics_count, 0);
    assert!(stream.topics.is_empty());
    assert_eq!(stream.size, 0);
    assert_eq!(stream.messages_count, 0);

    // 6. Get stream details by name
    let stream = client
        .get_stream(&Identifier::named(STREAM_NAME).unwrap())
        .await
        .unwrap()
        .expect("Failed to get stream");

    assert_eq!(stream.id, STREAM_ID);
    assert_eq!(stream.name, STREAM_NAME);

    // 7. Try to create the stream with the same ID but the different name and validate that it fails
    let create_stream_result = client
        .create_stream(&format!("{}-2", STREAM_NAME), Some(STREAM_ID))
        .await;
    assert!(create_stream_result.is_err());

    // 8. Try to create the stream with the same name but the different ID and validate that it fails
    let create_stream_result = client.create_stream(STREAM_NAME, Some(STREAM_ID + 1)).await;
    assert!(create_stream_result.is_err());

    // 9. Create the topic
    let topic = client
        .create_topic(
            &Identifier::numeric(STREAM_ID).unwrap(),
            TOPIC_NAME,
            PARTITIONS_COUNT,
            Default::default(),
            None,
            Some(TOPIC_ID),
            IggyExpiry::NeverExpire,
            MaxTopicSize::ServerDefault,
        )
        .await
        .unwrap();

    assert_eq!(topic.id, TOPIC_ID);
    assert_eq!(topic.name, TOPIC_NAME);

    // 10. Get topics and validate that created topic exists
    let topics = client
        .get_topics(&Identifier::numeric(STREAM_ID).unwrap())
        .await
        .unwrap();
    assert_eq!(topics.len(), 1);
    let topic = topics.first().unwrap();
    assert_eq!(topic.id, TOPIC_ID);
    assert_eq!(topic.name, TOPIC_NAME);
    assert_eq!(topic.partitions_count, PARTITIONS_COUNT);
    assert_eq!(topic.compression_algorithm, CompressionAlgorithm::default());
    assert_eq!(topic.size, 0);
    assert_eq!(topic.messages_count, 0);
    assert_eq!(topic.message_expiry, IggyExpiry::NeverExpire);
    assert_eq!(
        topic.max_topic_size,
        MaxTopicSize::from_str("10 GB").unwrap()
    );
    assert_eq!(topic.replication_factor, 1);

    // 11. Get topic details by ID
    let topic = client
        .get_topic(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
        )
        .await
        .unwrap()
        .expect("Failed to get topic");
    assert_eq!(topic.id, TOPIC_ID);
    assert_eq!(topic.name, TOPIC_NAME);
    assert_eq!(topic.partitions_count, PARTITIONS_COUNT);
    assert_eq!(topic.partitions.len(), PARTITIONS_COUNT as usize);
    assert_eq!(topic.size, 0);
    assert_eq!(topic.messages_count, 0);
    let mut id = 1;
    for topic_partition in topic.partitions {
        assert_eq!(topic_partition.id, id);
        assert_eq!(topic_partition.segments_count, 1);
        assert_eq!(topic_partition.size, 0);
        assert_eq!(topic_partition.current_offset, 0);
        assert_eq!(topic_partition.messages_count, 0);
        id += 1;
    }

    // 12. Get topic details by name
    let topic = client
        .get_topic(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::named(TOPIC_NAME).unwrap(),
        )
        .await
        .unwrap()
        .expect("Failed to get topic");
    assert_eq!(topic.id, TOPIC_ID);
    assert_eq!(topic.name, TOPIC_NAME);

    // 13. Get stream details and validate that created topic exists
    let stream = client
        .get_stream(&Identifier::numeric(STREAM_ID).unwrap())
        .await
        .unwrap()
        .expect("Failed to get stream");
    assert_eq!(stream.id, STREAM_ID);
    assert_eq!(stream.name, STREAM_NAME);
    assert_eq!(stream.topics_count, 1);
    assert_eq!(stream.topics.len(), 1);
    assert_eq!(stream.messages_count, 0);
    let stream_topic = stream.topics.first().unwrap();
    assert_eq!(stream_topic.id, topic.id);
    assert_eq!(stream_topic.name, topic.name);
    assert_eq!(stream_topic.partitions_count, topic.partitions_count);
    assert_eq!(stream_topic.size, 0);
    assert_eq!(stream_topic.messages_count, 0);

    // 15. Try to create the topic with the same ID but the different name and validate that it fails
    let create_topic_result = client
        .create_topic(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &format!("{}-2", TOPIC_NAME),
            PARTITIONS_COUNT,
            Default::default(),
            None,
            Some(TOPIC_ID),
            IggyExpiry::NeverExpire,
            MaxTopicSize::ServerDefault,
        )
        .await;
    assert!(create_topic_result.is_err());

    // 16. Try to create the topic with the different ID but the same name and validate that it fails
    let create_topic_result = client
        .create_topic(
            &Identifier::numeric(STREAM_ID).unwrap(),
            TOPIC_NAME,
            PARTITIONS_COUNT,
            Default::default(),
            None,
            Some(TOPIC_ID + 1),
            IggyExpiry::NeverExpire,
            MaxTopicSize::ServerDefault,
        )
        .await;
    assert!(create_topic_result.is_err());

    // 17. Send messages to the specific topic and partition
    let mut messages = create_messages();
    client
        .send_messages(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            &Partitioning::partition_id(PARTITION_ID),
            &mut messages,
        )
        .await
        .unwrap();

    // 18. Poll messages from the specific partition in topic
    let polled_messages = client
        .poll_messages(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            Some(PARTITION_ID),
            &consumer,
            &PollingStrategy::offset(0),
            MESSAGES_COUNT,
            false,
        )
        .await
        .unwrap();
    assert_eq!(polled_messages.messages.len() as u32, MESSAGES_COUNT);
    for i in 0..MESSAGES_COUNT {
        let offset = i as u64;
        let message = polled_messages.messages.get(i as usize).unwrap();
        assert_message(message, offset);
    }

    // 19. Messages should be also polled in the smaller batches
    let batches_count = 10;
    let batch_size = MESSAGES_COUNT / batches_count;
    for i in 0..batches_count {
        let start_offset = (i * batch_size) as u64;
        let polled_messages = client
            .poll_messages(
                &Identifier::numeric(STREAM_ID).unwrap(),
                &Identifier::numeric(TOPIC_ID).unwrap(),
                Some(PARTITION_ID),
                &consumer,
                &PollingStrategy::offset(start_offset),
                batch_size,
                false,
            )
            .await
            .unwrap();
        assert_eq!(polled_messages.messages.len() as u32, batch_size);
        for i in 0..batch_size as u64 {
            let offset = start_offset + i;
            let message = polled_messages.messages.get(i as usize).unwrap();
            assert_message(message, offset);
        }
    }

    // 20. Get topic details and validate the partition details
    let topic = client
        .get_topic(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::named(TOPIC_NAME).unwrap(),
        )
        .await
        .unwrap()
        .expect("Failed to get topic");
    assert_eq!(topic.id, TOPIC_ID);
    assert_eq!(topic.name, TOPIC_NAME);
    assert_eq!(topic.partitions_count, PARTITIONS_COUNT);
    assert_eq!(topic.partitions.len(), PARTITIONS_COUNT as usize);
    assert_eq!(topic.size, 55890);
    assert_eq!(topic.messages_count, MESSAGES_COUNT as u64);
    let topic_partition = topic.partitions.get((PARTITION_ID - 1) as usize).unwrap();
    assert_eq!(topic_partition.id, PARTITION_ID);
    assert_eq!(topic_partition.segments_count, 1);
    assert!(topic_partition.size > 0);
    assert_eq!(topic_partition.current_offset, (MESSAGES_COUNT - 1) as u64);
    assert_eq!(topic_partition.messages_count, MESSAGES_COUNT as u64);

    // 21. Ensure that messages do not exist in the second partition in the same topic
    let polled_messages = client
        .poll_messages(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            Some(PARTITION_ID + 1),
            &consumer,
            &PollingStrategy::offset(0),
            MESSAGES_COUNT,
            false,
        )
        .await
        .unwrap();
    assert!(polled_messages.messages.is_empty());

    // 22. Get the existing customer offset and ensure it's 0
    let offset = client
        .get_consumer_offset(
            &consumer,
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            Some(PARTITION_ID),
        )
        .await
        .unwrap()
        .expect("Failed to get consumer offset");
    assert_eq!(offset.partition_id, PARTITION_ID);
    assert_eq!(offset.current_offset, (MESSAGES_COUNT - 1) as u64);
    assert_eq!(offset.stored_offset, 0);

    // 23. Store the consumer offset
    let stored_offset = 10;
    client
        .store_consumer_offset(
            &consumer,
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            Some(PARTITION_ID),
            stored_offset,
        )
        .await
        .unwrap();

    // 24. Get the existing customer offset and ensure it's the previously stored value
    let offset = client
        .get_consumer_offset(
            &consumer,
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            Some(PARTITION_ID),
        )
        .await
        .unwrap()
        .expect("Failed to get consumer offset");
    assert_eq!(offset.partition_id, PARTITION_ID);
    assert_eq!(offset.current_offset, (MESSAGES_COUNT - 1) as u64);
    assert_eq!(offset.stored_offset, stored_offset);

    // 25. Poll messages from the specific partition in topic using next with auto commit
    let messages_count = 10;
    let polled_messages = client
        .poll_messages(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            Some(PARTITION_ID),
            &consumer,
            &PollingStrategy::next(),
            messages_count,
            true,
        )
        .await
        .unwrap();
    assert_eq!(polled_messages.messages.len() as u32, messages_count);
    let first_offset = polled_messages.messages.first().unwrap().offset;
    let last_offset = polled_messages.messages.last().unwrap().offset;
    let expected_last_offset = stored_offset + messages_count as u64;
    assert_eq!(first_offset, stored_offset + 1);
    assert_eq!(last_offset, expected_last_offset);

    // 26. Get the existing customer offset and ensure that auto commit during poll has worked
    let offset = client
        .get_consumer_offset(
            &consumer,
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            Some(PARTITION_ID),
        )
        .await
        .unwrap()
        .expect("Failed to get consumer offset");
    assert_eq!(offset.partition_id, PARTITION_ID);
    assert_eq!(offset.current_offset, (MESSAGES_COUNT - 1) as u64);
    assert_eq!(offset.stored_offset, expected_last_offset);

    // 27. Get the consumer groups and validate that there are no groups
    let consumer_groups = client
        .get_consumer_groups(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
        )
        .await
        .unwrap();

    assert!(consumer_groups.is_empty());

    // 28. Create the consumer group
    let consumer_group = client
        .create_consumer_group(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            CONSUMER_GROUP_NAME,
            Some(CONSUMER_GROUP_ID),
        )
        .await
        .unwrap();

    assert_eq!(consumer_group.id, CONSUMER_GROUP_ID);
    assert_eq!(consumer_group.name, CONSUMER_GROUP_NAME);

    // 29. Get the consumer groups and validate that there is one group
    let consumer_groups = client
        .get_consumer_groups(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(consumer_groups.len(), 1);
    let consumer_group = consumer_groups.first().unwrap();
    assert_eq!(consumer_group.id, CONSUMER_GROUP_ID);
    assert_eq!(consumer_group.partitions_count, PARTITIONS_COUNT);
    assert_eq!(consumer_group.members_count, 0);

    // 30. Get the consumer group details
    let consumer_group = client
        .get_consumer_group(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            &Identifier::numeric(CONSUMER_GROUP_ID).unwrap(),
        )
        .await
        .unwrap()
        .expect("Failed to get consumer group");

    assert_eq!(consumer_group.id, CONSUMER_GROUP_ID);
    assert_eq!(consumer_group.partitions_count, PARTITIONS_COUNT);
    assert_eq!(consumer_group.members_count, 0);
    assert!(consumer_group.members.is_empty());

    // 31. Join the consumer group and then leave it if the feature is available
    let result = client
        .join_consumer_group(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            &Identifier::numeric(CONSUMER_GROUP_ID).unwrap(),
        )
        .await;

    match result {
        Ok(_) => {
            let consumer_group = get_consumer_group(&client).await;
            assert_eq!(consumer_group.id, CONSUMER_GROUP_ID);
            assert_eq!(consumer_group.partitions_count, PARTITIONS_COUNT);
            assert_eq!(consumer_group.name, CONSUMER_GROUP_NAME);
            assert_eq!(consumer_group.members_count, 1);
            assert_eq!(consumer_group.members.len(), 1);
            let member = &consumer_group.members[0];
            assert_eq!(member.partitions_count, PARTITIONS_COUNT);

            let me = client.get_me().await.unwrap();
            assert!(me.client_id > 0);
            assert_eq!(me.consumer_groups_count, 1);
            assert_eq!(me.consumer_groups.len(), 1);
            let consumer_group = &me.consumer_groups[0];
            assert_eq!(consumer_group.stream_id, STREAM_ID);
            assert_eq!(consumer_group.topic_id, TOPIC_ID);
            assert_eq!(consumer_group.group_id, CONSUMER_GROUP_ID);

            leave_consumer_group(&client).await;

            let consumer_group = get_consumer_group(&client).await;
            assert_eq!(consumer_group.members_count, 0);
            assert!(consumer_group.members.is_empty());

            let me = client.get_me().await.unwrap();
            assert_eq!(me.consumer_groups_count, 0);
            assert!(me.consumer_groups.is_empty());
        }
        Err(e) => assert_eq!(e.as_code(), IggyError::FeatureUnavailable.as_code()),
    }

    // 32. Get the stats and validate that there is one stream
    let stats = client.get_stats().await.unwrap();
    assert!(!stats.hostname.is_empty());
    assert!(!stats.os_name.is_empty());
    assert!(!stats.os_version.is_empty());
    assert!(!stats.kernel_version.is_empty());
    assert_eq!(stats.streams_count, 1);
    assert_eq!(stats.topics_count, 1);
    assert_eq!(stats.partitions_count, PARTITIONS_COUNT);
    assert_eq!(stats.segments_count, PARTITIONS_COUNT);
    assert_eq!(stats.messages_count, MESSAGES_COUNT as u64);

    // 33. Delete the consumer group
    client
        .delete_consumer_group(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            &Identifier::numeric(CONSUMER_GROUP_ID).unwrap(),
        )
        .await
        .unwrap();

    // 34. Create new partitions and validate that the number of partitions is increased
    client
        .create_partitions(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            PARTITIONS_COUNT,
        )
        .await
        .unwrap();

    let topic = client
        .get_topic(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
        )
        .await
        .unwrap()
        .expect("Failed to get topic");

    assert_eq!(topic.partitions_count, 2 * PARTITIONS_COUNT);

    // 35. Delete the partitions and validate that the number of partitions is decreased
    client
        .delete_partitions(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            PARTITIONS_COUNT,
        )
        .await
        .unwrap();

    let topic = client
        .get_topic(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
        )
        .await
        .unwrap()
        .expect("Failed to get topic");

    assert_eq!(topic.partitions_count, PARTITIONS_COUNT);

    // 36. Update the existing topic and ensure it's updated
    let updated_topic_name = format!("{}-updated", TOPIC_NAME);
    let updated_message_expiry = 1000;
    let message_expiry_duration = updated_message_expiry.into();
    let updated_max_topic_size = MaxTopicSize::Custom(IggyByteSize::from_str("2 GB").unwrap());
    let updated_replication_factor = 5;

    client
        .update_topic(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            &updated_topic_name,
            CompressionAlgorithm::Gzip,
            Some(updated_replication_factor),
            IggyExpiry::ExpireDuration(message_expiry_duration),
            updated_max_topic_size,
        )
        .await
        .unwrap();

    let updated_topic = client
        .get_topic(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
        )
        .await
        .unwrap()
        .expect("Failed to get topic");

    assert_eq!(updated_topic.name, updated_topic_name);
    assert_eq!(
        updated_topic.message_expiry,
        IggyExpiry::ExpireDuration(message_expiry_duration)
    );
    assert_eq!(
        updated_topic.compression_algorithm,
        CompressionAlgorithm::Gzip
    );
    assert_eq!(updated_topic.max_topic_size, updated_max_topic_size);
    assert_eq!(updated_topic.replication_factor, updated_replication_factor);

    // 37. Purge the existing topic and ensure it has no messages
    client
        .purge_topic(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
        )
        .await
        .unwrap();

    let polled_messages = client
        .poll_messages(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            Some(PARTITION_ID),
            &consumer,
            &PollingStrategy::offset(0),
            MESSAGES_COUNT,
            false,
        )
        .await
        .unwrap();
    assert_eq!(polled_messages.current_offset, 0);
    assert!(polled_messages.messages.is_empty());

    // 38. Update the existing stream and ensure it's updated
    let updated_stream_name = format!("{}-updated", STREAM_NAME);

    client
        .update_stream(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &updated_stream_name,
        )
        .await
        .unwrap();

    let updated_stream = client
        .get_stream(&Identifier::numeric(STREAM_ID).unwrap())
        .await
        .unwrap()
        .expect("Failed to get stream");

    assert_eq!(updated_stream.name, updated_stream_name);

    // 39. Purge the existing stream and ensure it has no messages
    let mut messages = create_messages();
    client
        .send_messages(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            &Partitioning::partition_id(PARTITION_ID),
            &mut messages,
        )
        .await
        .unwrap();

    client
        .purge_stream(&Identifier::numeric(STREAM_ID).unwrap())
        .await
        .unwrap();

    let polled_messages = client
        .poll_messages(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            Some(PARTITION_ID),
            &consumer,
            &PollingStrategy::offset(0),
            MESSAGES_COUNT,
            false,
        )
        .await
        .unwrap();
    assert_eq!(polled_messages.current_offset, 0);
    assert!(polled_messages.messages.is_empty());

    // 40. Delete the existing topic and ensure it doesn't exist anymore
    client
        .delete_topic(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
        )
        .await
        .unwrap();
    let topics = client
        .get_topics(&Identifier::numeric(STREAM_ID).unwrap())
        .await
        .unwrap();
    assert!(topics.is_empty());

    // 41. Create the stream with automatically generated ID on the server
    let stream_name = format!("{}-auto", STREAM_NAME);
    let stream_id = STREAM_ID + 1;
    client.create_stream(&stream_name, None).await.unwrap();

    let stream = client
        .get_stream(&Identifier::numeric(stream_id).unwrap())
        .await
        .unwrap()
        .expect("Failed to get stream");

    assert_eq!(stream.id, stream_id);
    assert_eq!(stream.name, stream_name);

    // 42. Create the topic with automatically generated ID on the server
    let topic_name = format!("{}-auto", TOPIC_NAME);
    let topic_id = 1;
    client
        .create_topic(
            &Identifier::numeric(stream_id).unwrap(),
            &topic_name,
            PARTITIONS_COUNT,
            CompressionAlgorithm::default(),
            None,
            None,
            IggyExpiry::NeverExpire,
            MaxTopicSize::ServerDefault,
        )
        .await
        .unwrap();

    let topic = client
        .get_topic(
            &Identifier::numeric(stream_id).unwrap(),
            &Identifier::numeric(topic_id).unwrap(),
        )
        .await
        .unwrap()
        .expect("Failed to get topic");

    assert_eq!(topic.id, topic_id);
    assert_eq!(topic.name, topic_name);

    // 43. Delete the existing streams and ensure there's no streams left
    let streams = client.get_streams().await.unwrap();
    assert_eq!(streams.len(), 2);

    for stream in streams {
        client
            .delete_stream(&Identifier::numeric(stream.id).unwrap())
            .await
            .unwrap();
    }

    let streams = client.get_streams().await.unwrap();
    assert!(streams.is_empty());

    // 44. Get clients and ensure that there's 0 (HTTP) or 1 (TCP, QUIC) client
    let clients = client.get_clients().await.unwrap();

    assert!(clients.len() <= 1);

    assert_clean_system(&client).await;
}

fn assert_message(message: &PolledMessage, offset: u64) {
    let expected_payload = create_message_payload(offset);
    assert!(message.timestamp > 0);
    assert_eq!(message.offset, offset);
    assert_eq!(message.payload, expected_payload);
}

fn create_messages() -> Vec<Message> {
    let mut messages = Vec::new();
    for offset in 0..MESSAGES_COUNT {
        let id = (offset + 1) as u128;
        let payload = create_message_payload(offset as u64);
        messages.push(Message {
            id,
            length: payload.len() as u32,
            payload,
            headers: None,
        });
    }
    messages
}

fn create_message_payload(offset: u64) -> Bytes {
    Bytes::from(format!("message {}", offset))
}
