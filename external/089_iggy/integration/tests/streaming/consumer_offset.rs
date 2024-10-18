use crate::streaming::common::test_setup::TestSetup;
use iggy::consumer::ConsumerKind;
use server::configs::system::SystemConfig;
use server::streaming::partitions::partition::ConsumerOffset;
use server::streaming::storage::PartitionStorage;
use std::sync::Arc;
use tokio::fs;

#[tokio::test]
async fn should_persist_consumer_offsets_and_then_load_them_from_disk() {
    let setup = TestSetup::init().await;
    let storage = setup.storage.partition.as_ref();
    assert_persisted_offsets(&setup.config, storage, ConsumerKind::Consumer).await;
    assert_persisted_offsets(&setup.config, storage, ConsumerKind::ConsumerGroup).await;
}

async fn assert_persisted_offsets(
    config: &Arc<SystemConfig>,
    storage: &dyn PartitionStorage,
    kind: ConsumerKind,
) {
    let consumer_ids_count = 3;
    let offsets_count = 5;
    let path = match kind {
        ConsumerKind::Consumer => "consumer_offsets",
        ConsumerKind::ConsumerGroup => "consumer_group_offsets",
    };
    let path = format!("{}/{}", config.get_system_path(), path);
    fs::create_dir(&path).await.unwrap();
    for consumer_id in 1..=consumer_ids_count {
        let expected_offsets_count = consumer_id;
        for offset in 0..=offsets_count {
            let consumer_offset = ConsumerOffset::new(kind, consumer_id, offset, &path);
            assert_persisted_offset(&path, storage, &consumer_offset, expected_offsets_count).await;
        }
    }
}

async fn assert_persisted_offset(
    path: &str,
    storage: &dyn PartitionStorage,
    consumer_offset: &ConsumerOffset,
    expected_offsets_count: u32,
) {
    storage.save_consumer_offset(consumer_offset).await.unwrap();
    let consumer_offsets = storage
        .load_consumer_offsets(consumer_offset.kind, path)
        .await
        .unwrap();
    let expected_offsets_count = expected_offsets_count as usize;
    assert_eq!(consumer_offsets.len(), expected_offsets_count);
    let loaded_consumer_offset = consumer_offsets.get(expected_offsets_count - 1).unwrap();
    assert_eq!(loaded_consumer_offset, consumer_offset);
}
