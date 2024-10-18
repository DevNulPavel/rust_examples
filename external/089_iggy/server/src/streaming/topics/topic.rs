use crate::configs::system::SystemConfig;
use crate::streaming::partitions::partition::Partition;
use crate::streaming::polling_consumer::PollingConsumer;
use crate::streaming::storage::SystemStorage;
use crate::streaming::topics::consumer_group::ConsumerGroup;
use core::fmt;
use iggy::compression::compression_algorithm::CompressionAlgorithm;
use iggy::consumer::{Consumer, ConsumerKind};
use iggy::error::IggyError;
use iggy::locking::IggySharedMut;
use iggy::utils::byte_size::IggyByteSize;
use iggy::utils::expiry::IggyExpiry;
use iggy::utils::timestamp::IggyTimestamp;
use iggy::utils::topic_size::MaxTopicSize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

const ALMOST_FULL_THRESHOLD: f64 = 0.9;

#[derive(Debug)]
pub struct Topic {
    pub stream_id: u32,
    pub topic_id: u32,
    pub name: String,
    pub path: String,
    pub partitions_path: String,
    pub(crate) size_bytes: Arc<AtomicU64>,
    pub(crate) size_of_parent_stream: Arc<AtomicU64>,
    pub(crate) messages_count_of_parent_stream: Arc<AtomicU64>,
    pub(crate) messages_count: Arc<AtomicU64>,
    pub(crate) segments_count_of_parent_stream: Arc<AtomicU32>,
    pub(crate) config: Arc<SystemConfig>,
    pub(crate) partitions: HashMap<u32, IggySharedMut<Partition>>,
    pub(crate) storage: Arc<SystemStorage>,
    pub(crate) consumer_groups: HashMap<u32, RwLock<ConsumerGroup>>,
    pub(crate) consumer_groups_ids: HashMap<String, u32>,
    pub(crate) current_consumer_group_id: AtomicU32,
    pub(crate) current_partition_id: AtomicU32,
    pub message_expiry: IggyExpiry,
    pub compression_algorithm: CompressionAlgorithm,
    pub max_topic_size: MaxTopicSize,
    pub replication_factor: u8,
    pub created_at: IggyTimestamp,
}

impl Topic {
    #[allow(clippy::too_many_arguments)]
    pub fn empty(
        stream_id: u32,
        topic_id: u32,
        name: &str,
        size_of_parent_stream: Arc<AtomicU64>,
        messages_count_of_parent_stream: Arc<AtomicU64>,
        segments_count_of_parent_stream: Arc<AtomicU32>,
        config: Arc<SystemConfig>,
        storage: Arc<SystemStorage>,
    ) -> Topic {
        Topic::create(
            stream_id,
            topic_id,
            name,
            0,
            config,
            storage,
            size_of_parent_stream,
            messages_count_of_parent_stream,
            segments_count_of_parent_stream,
            IggyExpiry::NeverExpire,
            Default::default(),
            MaxTopicSize::ServerDefault,
            1,
        )
        .unwrap()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create(
        stream_id: u32,
        topic_id: u32,
        name: &str,
        partitions_count: u32,
        config: Arc<SystemConfig>,
        storage: Arc<SystemStorage>,
        size_of_parent_stream: Arc<AtomicU64>,
        messages_count_of_parent_stream: Arc<AtomicU64>,
        segments_count_of_parent_stream: Arc<AtomicU32>,
        message_expiry: IggyExpiry,
        compression_algorithm: CompressionAlgorithm,
        max_topic_size: MaxTopicSize,
        replication_factor: u8,
    ) -> Result<Topic, IggyError> {
        let path = config.get_topic_path(stream_id, topic_id);
        let partitions_path = config.get_partitions_path(stream_id, topic_id);
        let mut topic = Topic {
            stream_id,
            topic_id,
            name: name.to_string(),
            partitions: HashMap::new(),
            path,
            partitions_path,
            storage,
            size_bytes: Arc::new(AtomicU64::new(0)),
            size_of_parent_stream,
            messages_count_of_parent_stream,
            messages_count: Arc::new(AtomicU64::new(0)),
            segments_count_of_parent_stream,
            consumer_groups: HashMap::new(),
            consumer_groups_ids: HashMap::new(),
            current_consumer_group_id: AtomicU32::new(1),
            current_partition_id: AtomicU32::new(1),
            message_expiry: Topic::get_message_expiry(message_expiry, &config),
            max_topic_size: Topic::get_max_topic_size(max_topic_size, &config)?,
            compression_algorithm,
            replication_factor,
            config,
            created_at: IggyTimestamp::now(),
        };

        info!(
            "Received message expiry: {}, set expiry: {}",
            message_expiry, topic.message_expiry
        );

        topic.add_partitions(partitions_count)?;
        Ok(topic)
    }

    pub fn is_full(&self) -> bool {
        match self.max_topic_size {
            MaxTopicSize::Unlimited => false,
            MaxTopicSize::ServerDefault => false,
            MaxTopicSize::Custom(size) => {
                self.size_bytes.load(Ordering::SeqCst) >= size.as_bytes_u64()
            }
        }
    }

    pub fn is_almost_full(&self) -> bool {
        match self.max_topic_size {
            MaxTopicSize::Unlimited => false,
            MaxTopicSize::ServerDefault => false,
            MaxTopicSize::Custom(size) => {
                self.size_bytes.load(Ordering::SeqCst)
                    >= (size.as_bytes_u64() as f64 * ALMOST_FULL_THRESHOLD) as u64
            }
        }
    }

    pub fn is_unlimited(&self) -> bool {
        matches!(self.max_topic_size, MaxTopicSize::Unlimited)
    }

    pub fn get_size(&self) -> IggyByteSize {
        IggyByteSize::from(self.size_bytes.load(Ordering::SeqCst))
    }

    pub fn get_partitions(&self) -> Vec<IggySharedMut<Partition>> {
        self.partitions.values().cloned().collect()
    }

    pub fn get_partition(&self, partition_id: u32) -> Result<IggySharedMut<Partition>, IggyError> {
        match self.partitions.get(&partition_id) {
            Some(partition_arc) => Ok(partition_arc.clone()),
            None => Err(IggyError::PartitionNotFound(
                partition_id,
                self.topic_id,
                self.stream_id,
            )),
        }
    }

    pub async fn resolve_consumer_with_partition_id(
        &self,
        consumer: &Consumer,
        client_id: u32,
        partition_id: Option<u32>,
        calculate_partition_id: bool,
    ) -> Result<(PollingConsumer, u32), IggyError> {
        match consumer.kind {
            ConsumerKind::Consumer => {
                let partition_id = partition_id.unwrap_or(1);
                Ok((
                    PollingConsumer::consumer(&consumer.id, partition_id),
                    partition_id,
                ))
            }
            ConsumerKind::ConsumerGroup => {
                let consumer_group = self.get_consumer_group(&consumer.id)?.read().await;
                if let Some(partition_id) = partition_id {
                    return Ok((
                        PollingConsumer::consumer_group(consumer_group.group_id, client_id),
                        partition_id,
                    ));
                }

                let partition_id = if calculate_partition_id {
                    consumer_group.calculate_partition_id(client_id).await?
                } else {
                    consumer_group.get_current_partition_id(client_id).await?
                };
                Ok((
                    PollingConsumer::consumer_group(consumer_group.group_id, client_id),
                    partition_id,
                ))
            }
        }
    }

    pub fn get_max_topic_size(
        max_topic_size: MaxTopicSize,
        config: &SystemConfig,
    ) -> Result<MaxTopicSize, IggyError> {
        match max_topic_size {
            MaxTopicSize::ServerDefault => Ok(config.topic.max_size),
            _ => {
                if max_topic_size.as_bytes_u64() >= config.segment.size.as_bytes_u64() {
                    Ok(max_topic_size)
                } else {
                    Err(IggyError::InvalidTopicSize(
                        max_topic_size,
                        config.segment.size,
                    ))
                }
            }
        }
    }

    pub fn get_message_expiry(message_expiry: IggyExpiry, config: &SystemConfig) -> IggyExpiry {
        match message_expiry {
            IggyExpiry::ServerDefault => config.segment.message_expiry,
            _ => message_expiry,
        }
    }
}

impl fmt::Display for Topic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ID: {}, ", self.topic_id)?;
        write!(f, "stream ID: {}, ", self.stream_id)?;
        write!(f, "name: {}, ", self.name)?;
        write!(f, "path: {}, ", self.path)?;
        write!(f, "partitions count: {}, ", self.partitions.len())?;
        write!(f, "message expiry: {}, ", self.message_expiry)?;
        write!(f, "max topic size: {}, ", self.max_topic_size)?;
        write!(f, "replication factor: {}, ", self.replication_factor)
    }
}

#[cfg(test)]
mod tests {
    use iggy::locking::IggySharedMutFn;
    use std::str::FromStr;

    use super::*;
    use crate::streaming::storage::tests::get_test_system_storage;

    #[tokio::test]
    async fn should_be_created_given_valid_parameters() {
        let storage = Arc::new(get_test_system_storage());
        let stream_id = 1;
        let topic_id = 2;
        let name = "test";
        let partitions_count = 3;
        let message_expiry = IggyExpiry::NeverExpire;
        let compression_algorithm = CompressionAlgorithm::None;
        let max_topic_size = MaxTopicSize::Custom(IggyByteSize::from_str("2 GB").unwrap());
        let replication_factor = 1;
        let config = Arc::new(SystemConfig::default());
        let path = config.get_topic_path(stream_id, topic_id);
        let size_of_parent_stream = Arc::new(AtomicU64::new(0));
        let messages_count_of_parent_stream = Arc::new(AtomicU64::new(0));
        let segments_count_of_parent_stream = Arc::new(AtomicU32::new(0));

        let topic = Topic::create(
            stream_id,
            topic_id,
            name,
            partitions_count,
            config,
            storage,
            messages_count_of_parent_stream,
            size_of_parent_stream,
            segments_count_of_parent_stream,
            message_expiry,
            compression_algorithm,
            max_topic_size,
            replication_factor,
        )
        .unwrap();

        assert_eq!(topic.stream_id, stream_id);
        assert_eq!(topic.topic_id, topic_id);
        assert_eq!(topic.path, path);
        assert_eq!(topic.name, name);
        assert_eq!(topic.partitions.len(), partitions_count as usize);
        assert_eq!(topic.message_expiry, message_expiry);

        for (id, partition) in topic.partitions {
            let partition = partition.read().await;
            assert_eq!(partition.stream_id, stream_id);
            assert_eq!(partition.topic_id, topic.topic_id);
            assert_eq!(partition.partition_id, id);
            assert_eq!(partition.segments.len(), 1);
        }
    }
}
