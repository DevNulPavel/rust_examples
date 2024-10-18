use clap::{Args, Subcommand};
use iggy::identifier::Identifier;

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum ConsumerOffsetAction {
    /// Retrieve the offset of a consumer for a given partition from the server
    ///
    /// Consumer ID can be specified as a consumer name or ID
    /// Stream ID can be specified as a stream name or ID
    /// Topic ID can be specified as a topic name or ID
    ///
    /// Examples:
    ///  iggy consumer-offset get 1 3 5 1
    ///  iggy consumer-offset get consumer stream 5 1
    ///  iggy consumer-offset get 1 3 topic 1
    ///  iggy consumer-offset get consumer stream 5 1
    ///  iggy consumer-offset get consumer 3 topic 1
    ///  iggy consumer-offset get 1 stream topic 1
    ///  iggy consumer-offset get consumer stream topic 1
    #[clap(verbatim_doc_comment, visible_alias = "g")]
    Get(ConsumerOffsetGetArgs),
    /// Set the offset of a consumer for a given partition on the server
    ///
    /// Consumer ID can be specified as a consumer name or ID
    /// Stream ID can be specified as a stream name or ID
    /// Topic ID can be specified as a topic name or ID
    ///
    /// Examples:
    ///  iggy consumer-offset set 1 3 5 1 100
    ///  iggy consumer-offset set consumer 3 5 1 100
    ///  iggy consumer-offset set 1 stream 5 1 100
    ///  iggy consumer-offset set 1 3 topic 1 100
    ///  iggy consumer-offset set consumer stream 5 1 100
    ///  iggy consumer-offset set consumer 3 topic 1 100
    ///  iggy consumer-offset set 1 stream topic 1 100
    ///  iggy consumer-offset set consumer stream topic 1 100
    #[clap(verbatim_doc_comment, visible_alias = "s")]
    Set(ConsumerOffsetSetArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ConsumerOffsetGetArgs {
    /// Regular consumer for which the offset is retrieved
    ///
    /// Consumer ID can be specified as a consumer name or ID
    #[clap(verbatim_doc_comment)]
    #[arg(value_parser = clap::value_parser!(Identifier))]
    pub(crate) consumer_id: Identifier,
    /// Stream ID for which consumer offset is retrieved
    ///
    /// Stream ID can be specified as a stream name or ID
    #[arg(value_parser = clap::value_parser!(Identifier))]
    pub(crate) stream_id: Identifier,
    /// Topic ID for which consumer offset is retrieved
    ///
    /// Topic ID can be specified as a topic name or ID
    #[arg(value_parser = clap::value_parser!(Identifier))]
    pub(crate) topic_id: Identifier,
    /// Partitions ID for which consumer offset is retrieved
    #[arg(value_parser = clap::value_parser!(u32).range(1..))]
    pub(crate) partition_id: u32,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ConsumerOffsetSetArgs {
    /// Regular consumer for which the offset is set
    ///
    /// Consumer ID can be specified as a consumer name or ID
    #[clap(verbatim_doc_comment)]
    #[arg(value_parser = clap::value_parser!(Identifier))]
    pub(crate) consumer_id: Identifier,
    /// Stream ID for which consumer offset is set
    ///
    /// Stream ID can be specified as a stream name or ID
    #[arg(value_parser = clap::value_parser!(Identifier))]
    pub(crate) stream_id: Identifier,
    /// Topic ID for which consumer offset is set
    ///
    /// Topic ID can be specified as a topic name or ID
    #[arg(value_parser = clap::value_parser!(Identifier))]
    pub(crate) topic_id: Identifier,
    /// Partitions ID for which consumer offset is set
    #[arg(value_parser = clap::value_parser!(u32).range(1..))]
    pub(crate) partition_id: u32,
    /// Offset to set
    pub(crate) offset: u64,
}
