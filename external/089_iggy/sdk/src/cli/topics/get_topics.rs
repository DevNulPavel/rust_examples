use crate::cli_command::{CliCommand, PRINT_TARGET};
use crate::client::Client;
use crate::identifier::Identifier;
use crate::topics::get_topics::GetTopics;
use crate::utils::expiry::IggyExpiry;
use anyhow::Context;
use async_trait::async_trait;
use comfy_table::Table;
use std::fmt::{self, Display, Formatter};
use tracing::{event, Level};

pub enum GetTopicsOutput {
    Table,
    List,
}

impl Display for GetTopicsOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            GetTopicsOutput::Table => write!(f, "table"),
            GetTopicsOutput::List => write!(f, "list"),
        }?;

        Ok(())
    }
}

pub struct GetTopicsCmd {
    get_topics: GetTopics,
    output: GetTopicsOutput,
}

impl GetTopicsCmd {
    pub fn new(stream_id: Identifier, output: GetTopicsOutput) -> Self {
        Self {
            get_topics: GetTopics { stream_id },
            output,
        }
    }
}

#[async_trait]
impl CliCommand for GetTopicsCmd {
    fn explain(&self) -> String {
        format!(
            "list topics from stream with ID: {} in {} mode",
            self.get_topics.stream_id, self.output
        )
    }

    async fn execute_cmd(&mut self, client: &dyn Client) -> anyhow::Result<(), anyhow::Error> {
        let topics = client
            .get_topics(&self.get_topics.stream_id)
            .await
            .with_context(|| {
                format!(
                    "Problem getting topics from stream {}",
                    self.get_topics.stream_id
                )
            })?;

        match self.output {
            GetTopicsOutput::Table => {
                let mut table = Table::new();

                table.set_header(vec![
                    "ID",
                    "Created",
                    "Name",
                    "Size (B)",
                    "Max Topic Size (B)",
                    "Compression",
                    "Message Expiry (s)",
                    "Messages Count",
                    "Partitions Count",
                ]);

                topics.iter().for_each(|topic| {
                    table.add_row(vec![
                        format!("{}", topic.id),
                        topic.created_at.to_utc_string("%Y-%m-%d %H:%M:%S"),
                        topic.name.clone(),
                        format!("{}", topic.size),
                        format!("{}", topic.max_topic_size),
                        topic.compression_algorithm.to_string(),
                        match topic.message_expiry {
                            IggyExpiry::NeverExpire => String::from("unlimited"),
                            IggyExpiry::ServerDefault => String::from("server_default"),
                            IggyExpiry::ExpireDuration(value) => format!("{}", value),
                        },
                        format!("{}", topic.messages_count),
                        format!("{}", topic.partitions_count),
                    ]);
                });

                event!(target: PRINT_TARGET, Level::INFO, "{table}");
            }
            GetTopicsOutput::List => {
                topics.iter().for_each(|topic| {
                    event!(target: PRINT_TARGET, Level::INFO,
                            "{}|{}|{}|{}|{}|{}|{}|{}|{}",
                            topic.id,
                            topic.created_at.to_utc_string("%Y-%m-%d %H:%M:%S"),
                            topic.name,
                            topic.size,
                            topic.max_topic_size,
                            topic.compression_algorithm.to_string(),
                            match topic.message_expiry {
                    IggyExpiry::NeverExpire => String::from("unlimited"),
                    IggyExpiry::ServerDefault => String::from("server_default"),
                    IggyExpiry::ExpireDuration(value) => format!("{}", value),
                            },
                            topic.messages_count,
                            topic.partitions_count
                        );
                });
            }
        }

        Ok(())
    }
}
