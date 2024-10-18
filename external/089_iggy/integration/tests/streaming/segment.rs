use crate::streaming::common::test_setup::TestSetup;
use bytes::Bytes;
use iggy::bytes_serializable::BytesSerializable;
use iggy::models::messages::{MessageState, PolledMessage};
use iggy::utils::expiry::IggyExpiry;
use iggy::utils::{checksum, timestamp::IggyTimestamp};
use server::streaming::models::messages::RetainedMessage;
use server::streaming::segments::segment;
use server::streaming::segments::segment::{INDEX_EXTENSION, LOG_EXTENSION, TIME_INDEX_EXTENSION};
use server::streaming::sizeable::Sizeable;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::fs;

#[tokio::test]
async fn should_persist_segment() {
    let setup = TestSetup::init().await;
    let stream_id = 1;
    let topic_id = 2;
    let partition_id = 3;
    let start_offsets = get_start_offsets();
    for start_offset in start_offsets {
        let segment = segment::Segment::create(
            stream_id,
            topic_id,
            partition_id,
            start_offset,
            setup.config.clone(),
            setup.storage.clone(),
            IggyExpiry::NeverExpire,
            Arc::new(AtomicU64::new(0)),
            Arc::new(AtomicU64::new(0)),
            Arc::new(AtomicU64::new(0)),
            Arc::new(AtomicU64::new(0)),
            Arc::new(AtomicU64::new(0)),
            Arc::new(AtomicU64::new(0)),
        );

        setup
            .create_partition_directory(stream_id, topic_id, partition_id)
            .await;
        segment.persist().await.unwrap();
        assert_persisted_segment(
            &setup
                .config
                .get_partition_path(stream_id, topic_id, partition_id),
            start_offset,
        )
        .await;
    }
}

#[tokio::test]
async fn should_load_existing_segment_from_disk() {
    let setup = TestSetup::init().await;
    let stream_id = 1;
    let topic_id = 2;
    let partition_id = 3;
    let start_offsets = get_start_offsets();
    for start_offset in start_offsets {
        let segment = segment::Segment::create(
            stream_id,
            topic_id,
            partition_id,
            start_offset,
            setup.config.clone(),
            setup.storage.clone(),
            IggyExpiry::NeverExpire,
            Arc::new(AtomicU64::new(0)),
            Arc::new(AtomicU64::new(0)),
            Arc::new(AtomicU64::new(0)),
            Arc::new(AtomicU64::new(0)),
            Arc::new(AtomicU64::new(0)),
            Arc::new(AtomicU64::new(0)),
        );
        setup
            .create_partition_directory(stream_id, topic_id, partition_id)
            .await;
        segment.persist().await.unwrap();
        assert_persisted_segment(
            &setup
                .config
                .get_partition_path(stream_id, topic_id, partition_id),
            start_offset,
        )
        .await;

        let mut loaded_segment = segment::Segment::create(
            stream_id,
            topic_id,
            partition_id,
            start_offset,
            setup.config.clone(),
            setup.storage.clone(),
            IggyExpiry::NeverExpire,
            Arc::new(AtomicU64::new(0)),
            Arc::new(AtomicU64::new(0)),
            Arc::new(AtomicU64::new(0)),
            Arc::new(AtomicU64::new(0)),
            Arc::new(AtomicU64::new(0)),
            Arc::new(AtomicU64::new(0)),
        );
        loaded_segment.load().await.unwrap();
        let loaded_messages = loaded_segment.get_messages(0, 10).await.unwrap();

        assert_eq!(loaded_segment.partition_id, segment.partition_id);
        assert_eq!(loaded_segment.start_offset, segment.start_offset);
        assert_eq!(loaded_segment.current_offset, segment.current_offset);
        assert_eq!(loaded_segment.end_offset, segment.end_offset);
        assert_eq!(loaded_segment.size_bytes, segment.size_bytes);
        assert_eq!(loaded_segment.is_closed, segment.is_closed);
        assert_eq!(loaded_segment.log_path, segment.log_path);
        assert_eq!(loaded_segment.index_path, segment.index_path);
        assert_eq!(loaded_segment.time_index_path, segment.time_index_path);
        assert!(loaded_messages.is_empty());
    }
}

#[tokio::test]
async fn should_persist_and_load_segment_with_messages() {
    let setup = TestSetup::init().await;
    let stream_id = 1;
    let topic_id = 2;
    let partition_id = 3;
    let start_offset = 0;
    let mut segment = segment::Segment::create(
        stream_id,
        topic_id,
        partition_id,
        start_offset,
        setup.config.clone(),
        setup.storage.clone(),
        IggyExpiry::NeverExpire,
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
    );

    setup
        .create_partition_directory(stream_id, topic_id, partition_id)
        .await;
    segment.persist().await.unwrap();
    assert_persisted_segment(
        &setup
            .config
            .get_partition_path(stream_id, topic_id, partition_id),
        start_offset,
    )
    .await;
    let messages_count = 10;
    let mut messages = Vec::new();
    let mut batch_size = 0u64;
    for i in 0..messages_count {
        let message = create_message(i, "test", IggyTimestamp::now());

        let retained_message = Arc::new(RetainedMessage {
            id: message.id,
            offset: message.offset,
            timestamp: message.timestamp,
            checksum: message.checksum,
            message_state: message.state,
            headers: message.headers.map(|headers| headers.to_bytes()),
            payload: message.payload.clone(),
        });
        batch_size += retained_message.get_size_bytes() as u64;
        messages.push(retained_message);
    }

    segment
        .append_batch(batch_size, messages_count as u32, &messages)
        .await
        .unwrap();
    segment.persist_messages().await.unwrap();
    let mut loaded_segment = segment::Segment::create(
        stream_id,
        topic_id,
        partition_id,
        start_offset,
        setup.config.clone(),
        setup.storage.clone(),
        IggyExpiry::NeverExpire,
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
    );
    loaded_segment.load().await.unwrap();
    let messages = loaded_segment
        .get_messages(0, messages_count as u32)
        .await
        .unwrap();
    assert_eq!(messages.len(), messages_count as usize);
}

#[tokio::test]
async fn given_all_expired_messages_segment_should_be_expired() {
    let setup = TestSetup::init().await;
    let stream_id = 1;
    let topic_id = 2;
    let partition_id = 3;
    let start_offset = 0;
    let message_expiry_ms = 1000;
    let message_expiry = 1000u64.into();
    let mut segment = segment::Segment::create(
        stream_id,
        topic_id,
        partition_id,
        start_offset,
        setup.config.clone(),
        setup.storage.clone(),
        message_expiry,
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
    );

    setup
        .create_partition_directory(stream_id, topic_id, partition_id)
        .await;
    segment.persist().await.unwrap();
    assert_persisted_segment(
        &setup
            .config
            .get_partition_path(stream_id, topic_id, partition_id),
        start_offset,
    )
    .await;
    let messages_count = 10;
    let now = IggyTimestamp::now();
    let mut expired_timestamp = (now.as_micros() - 2 * message_expiry_ms).into();
    let mut batch_size = 0u64;
    let mut messages = Vec::new();
    for i in 0..messages_count {
        let message = create_message(i, "test", expired_timestamp);
        expired_timestamp = (expired_timestamp.as_micros() + 1).into();

        let retained_message = Arc::new(RetainedMessage {
            id: message.id,
            offset: message.offset,
            timestamp: message.timestamp,
            checksum: message.checksum,
            message_state: message.state,
            headers: message.headers.map(|headers| headers.to_bytes()),
            payload: message.payload.clone(),
        });
        batch_size += retained_message.get_size_bytes() as u64;
        messages.push(retained_message);
    }
    segment
        .append_batch(batch_size, messages_count as u32, &messages)
        .await
        .unwrap();
    segment.persist_messages().await.unwrap();

    segment.is_closed = true;
    let is_expired = segment.is_expired(now).await;
    assert!(is_expired);
}

#[tokio::test]
async fn given_at_least_one_not_expired_message_segment_should_not_be_expired() {
    let setup = TestSetup::init().await;
    let stream_id = 1;
    let topic_id = 2;
    let partition_id = 3;
    let start_offset = 0;
    let message_expiry_ms = 1000;
    let message_expiry = message_expiry_ms.into();
    let mut segment = segment::Segment::create(
        stream_id,
        topic_id,
        partition_id,
        start_offset,
        setup.config.clone(),
        setup.storage.clone(),
        message_expiry,
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
        Arc::new(AtomicU64::new(0)),
    );

    setup
        .create_partition_directory(stream_id, topic_id, partition_id)
        .await;
    segment.persist().await.unwrap();
    assert_persisted_segment(
        &setup
            .config
            .get_partition_path(stream_id, topic_id, partition_id),
        start_offset,
    )
    .await;
    let now = IggyTimestamp::now();
    let expired_timestamp = now.as_micros() - 2 * message_expiry_ms;
    let not_expired_timestamp = now.as_micros() - message_expiry_ms + 1;
    let expired_message = create_message(0, "test", expired_timestamp.into());
    let not_expired_message = create_message(1, "test", not_expired_timestamp.into());

    let expired_retained_message = Arc::new(RetainedMessage {
        id: expired_message.id,
        offset: expired_message.offset,
        timestamp: expired_message.timestamp,
        checksum: expired_message.checksum,
        message_state: expired_message.state,
        headers: expired_message.headers.map(|headers| headers.to_bytes()),
        payload: expired_message.payload.clone(),
    });
    let mut expired_messages = Vec::new();
    let expired_message_size = expired_retained_message.get_size_bytes() as u64;
    expired_messages.push(expired_retained_message);

    let mut not_expired_messages = Vec::new();
    let not_expired_retained_message = Arc::new(RetainedMessage {
        id: not_expired_message.id,
        offset: not_expired_message.offset,
        timestamp: not_expired_message.timestamp,
        checksum: not_expired_message.checksum,
        message_state: not_expired_message.state,
        headers: not_expired_message
            .headers
            .map(|headers| headers.to_bytes()),
        payload: not_expired_message.payload.clone(),
    });
    let not_expired_message_size = not_expired_retained_message.get_size_bytes() as u64;
    not_expired_messages.push(not_expired_retained_message);

    segment
        .append_batch(expired_message_size, 1, &expired_messages)
        .await
        .unwrap();
    segment
        .append_batch(not_expired_message_size, 1, &not_expired_messages)
        .await
        .unwrap();
    segment.persist_messages().await.unwrap();

    let is_expired = segment.is_expired(now).await;
    assert!(!is_expired);
}

async fn assert_persisted_segment(partition_path: &str, start_offset: u64) {
    let segment_path = format!("{}/{:0>20}", partition_path, start_offset);
    let log_path = format!("{}.{}", segment_path, LOG_EXTENSION);
    let index_path = format!("{}.{}", segment_path, INDEX_EXTENSION);
    let time_index_path = format!("{}.{}", segment_path, TIME_INDEX_EXTENSION);
    assert!(fs::metadata(&log_path).await.is_ok());
    assert!(fs::metadata(&index_path).await.is_ok());
    assert!(fs::metadata(&time_index_path).await.is_ok());
}

fn create_message(offset: u64, payload: &str, timestamp: IggyTimestamp) -> PolledMessage {
    let payload = Bytes::from(payload.to_string());
    let checksum = checksum::calculate(payload.as_ref());
    PolledMessage::create(
        offset,
        MessageState::Available,
        timestamp,
        0,
        payload,
        checksum,
        None,
    )
}

fn get_start_offsets() -> Vec<u64> {
    vec![
        0, 1, 2, 9, 10, 99, 100, 110, 200, 1000, 1234, 12345, 100000, 9999999,
    ]
}
