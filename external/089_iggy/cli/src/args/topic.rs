use crate::args::common::ListMode;
use clap::{Args, Subcommand};
use iggy::compression::compression_algorithm::CompressionAlgorithm;
use iggy::identifier::Identifier;
use iggy::utils::expiry::IggyExpiry;
use iggy::utils::topic_size::MaxTopicSize;

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum TopicAction {
    /// Create topic with given name, number of partitions, compression algorithm and expiry time for given stream ID
    ///
    /// Stream ID can be specified as a stream name or ID
    /// If topic ID is not provided then the server will automatically assign it
    ///
    /// Examples
    ///  iggy topic create 1 sensor1 2 gzip 15days
    ///  iggy topic create prod sensor2 2 none
    ///  iggy topic create test debugs 2 gzip 1day 1hour 1min 1sec
    ///  iggy topic create -t 3 1 sensor3 2 none unlimited
    #[clap(verbatim_doc_comment, visible_alias = "c")]
    Create(TopicCreateArgs),
    /// Delete topic with given ID in given stream ID
    ///
    /// Stream ID can be specified as a stream name or ID
    /// Topic ID can be specified as a topic name or ID
    ///
    /// Examples
    ///  iggy topic delete 1 1
    ///  iggy topic delete prod 2
    ///  iggy topic delete test debugs
    ///  iggy topic delete 2 debugs
    #[clap(verbatim_doc_comment, visible_alias = "d")]
    Delete(TopicDeleteArgs),
    /// Update topic name, compression algorithm and message expiry time for given topic ID in given stream ID
    ///
    /// Stream ID can be specified as a stream name or ID
    /// Topic ID can be specified as a topic name or ID
    ///
    /// Examples
    ///  iggy update 1 1 sensor3 none
    ///  iggy update prod sensor3 old-sensor none
    ///  iggy update test debugs ready gzip 15days
    ///  iggy update 1 1 new-name gzip
    ///  iggy update 1 2 new-name none 1day 1hour 1min 1sec
    #[clap(verbatim_doc_comment, visible_alias = "u")]
    Update(TopicUpdateArgs),
    /// Get topic detail for given topic ID and stream ID
    ///
    /// Stream ID can be specified as a stream name or ID
    /// Topic ID can be specified as a topic name or ID
    ///
    /// Examples
    ///  iggy topic get 1 1
    ///  iggy topic get prod 2
    ///  iggy topic get test debugs
    ///  iggy topic get 2 debugs
    #[clap(verbatim_doc_comment, visible_alias = "g")]
    Get(TopicGetArgs),
    /// List all topics in given stream ID
    ///
    /// Stream ID can be specified as a stream name or ID
    ///
    /// Examples
    ///  iggy topic list 1
    ///  iggy topic list prod
    #[clap(verbatim_doc_comment, visible_alias = "l")]
    List(TopicListArgs),
    /// Purge topic with given ID in given stream ID
    ///
    /// Command removes all messages from given topic
    /// Stream ID can be specified as a stream name or ID
    /// Topic ID can be specified as a topic name or ID
    ///
    /// Examples
    ///  iggy topic purge 1 1
    ///  iggy topic purge prod 2
    ///  iggy topic purge test debugs
    ///  iggy topic purge 2 debugs
    #[clap(verbatim_doc_comment, visible_alias = "p")]
    Purge(TopicPurgeArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct TopicCreateArgs {
    /// Stream ID to create topic
    ///
    /// Stream ID can be specified as a stream name or ID
    #[arg(value_parser = clap::value_parser!(Identifier))]
    pub(crate) stream_id: Identifier,
    /// Name of the topic
    pub(crate) name: String,
    /// Topic ID to create
    #[clap(short, long)]
    pub(crate) topic_id: Option<u32>,
    /// Number of partitions inside the topic
    pub(crate) partitions_count: u32,
    /// Compression algorithm for the topic, set to "none" for no compression
    #[arg(value_parser = clap::value_parser!(CompressionAlgorithm), verbatim_doc_comment)]
    pub(crate) compression_algorithm: CompressionAlgorithm,
    /// Max topic size
    ///
    /// ("unlimited" or skipping parameter disables max topic size functionality in topic)
    /// Can't be lower than segment size in the config.
    #[arg(short, long, default_value = "unlimited", verbatim_doc_comment)]
    pub(crate) max_topic_size: MaxTopicSize,
    /// Replication factor for the topic
    #[arg(short, long, default_value = "1")]
    pub(crate) replication_factor: u8,
    /// Message expiry time in human-readable format like 15days 2min 2s
    ///
    /// ("unlimited" or skipping parameter disables message expiry functionality in topic)
    #[arg(value_parser = clap::value_parser!(IggyExpiry), verbatim_doc_comment)]
    pub(crate) message_expiry: Vec<IggyExpiry>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct TopicDeleteArgs {
    /// Stream ID to delete topic
    ///
    /// Stream ID can be specified as a stream name or ID
    #[arg(value_parser = clap::value_parser!(Identifier))]
    pub(crate) stream_id: Identifier,
    /// Topic ID to delete
    ///
    /// Topic ID can be specified as a topic name or ID
    #[arg(value_parser = clap::value_parser!(Identifier))]
    pub(crate) topic_id: Identifier,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct TopicUpdateArgs {
    /// Stream ID to update topic
    ///
    /// Stream ID can be specified as a stream name or ID
    #[arg(value_parser = clap::value_parser!(Identifier))]
    pub(crate) stream_id: Identifier,
    /// Topic ID to update
    ///
    /// Topic ID can be specified as a topic name or ID
    #[arg(value_parser = clap::value_parser!(Identifier))]
    pub(crate) topic_id: Identifier,
    /// New name for the topic
    pub(crate) name: String,
    /// Compression algorithm for the topic, set to "none" for no compression
    #[arg(value_parser = clap::value_parser!(CompressionAlgorithm), verbatim_doc_comment)]
    pub(crate) compression_algorithm: CompressionAlgorithm,
    /// New max topic size
    ///
    /// ("unlimited" or skipping parameter causes removal of max topic size parameter in topic)
    /// Can't be lower than segment size in the config.
    #[arg(short, long, default_value = "unlimited", verbatim_doc_comment)]
    pub(crate) max_topic_size: MaxTopicSize,
    #[arg(short, long, default_value = "1")]
    /// New replication factor for the topic
    pub(crate) replication_factor: u8,
    /// New message expiry time in human-readable format like 15days 2min 2s
    ///
    /// ("unlimited" or skipping parameter causes removal of expiry parameter in topic)
    #[arg(value_parser = clap::value_parser!(IggyExpiry), verbatim_doc_comment)]
    pub(crate) message_expiry: Vec<IggyExpiry>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct TopicGetArgs {
    /// Stream ID to get topic
    ///
    /// Stream ID can be specified as a stream name or ID
    #[arg(value_parser = clap::value_parser!(Identifier))]
    pub(crate) stream_id: Identifier,
    /// Topic ID to get
    ///
    /// Topic ID can be specified as a topic name or ID
    #[arg(value_parser = clap::value_parser!(Identifier))]
    pub(crate) topic_id: Identifier,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct TopicListArgs {
    /// Stream ID to list topics
    ///
    /// Stream ID can be specified as a stream name or ID
    #[arg(value_parser = clap::value_parser!(Identifier))]
    pub(crate) stream_id: Identifier,

    /// List mode (table or list)
    #[clap(short, long, value_enum, default_value_t = ListMode::Table)]
    pub(crate) list_mode: ListMode,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct TopicPurgeArgs {
    /// Stream ID to purge topic
    ///
    /// Stream ID can be specified as a stream name or ID
    #[arg(value_parser = clap::value_parser!(Identifier))]
    pub(crate) stream_id: Identifier,
    /// Topic ID to purge
    ///
    /// Topic ID can be specified as a topic name or ID
    #[arg(value_parser = clap::value_parser!(Identifier))]
    pub(crate) topic_id: Identifier,
}
