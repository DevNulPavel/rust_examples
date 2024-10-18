use crate::cli_command::{CliCommand, PRINT_TARGET};
use crate::client::Client;
use crate::consumer_groups::get_consumer_groups::GetConsumerGroups;
use crate::identifier::Identifier;
use anyhow::Context;
use async_trait::async_trait;
use comfy_table::Table;
use std::fmt::{self, Display, Formatter};
use tracing::{event, Level};

pub enum GetConsumerGroupsOutput {
    Table,
    List,
}

impl Display for GetConsumerGroupsOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            GetConsumerGroupsOutput::Table => write!(f, "table"),
            GetConsumerGroupsOutput::List => write!(f, "list"),
        }?;

        Ok(())
    }
}

pub struct GetConsumerGroupsCmd {
    get_consumer_groups: GetConsumerGroups,
    output: GetConsumerGroupsOutput,
}

impl GetConsumerGroupsCmd {
    pub fn new(
        stream_id: Identifier,
        topic_id: Identifier,
        output: GetConsumerGroupsOutput,
    ) -> Self {
        Self {
            get_consumer_groups: GetConsumerGroups {
                stream_id,
                topic_id,
            },
            output,
        }
    }
}

#[async_trait]
impl CliCommand for GetConsumerGroupsCmd {
    fn explain(&self) -> String {
        format!(
            "list consumer groups for stream with ID: {} and topic with ID: {} in {} mode",
            self.get_consumer_groups.stream_id, self.get_consumer_groups.topic_id, self.output
        )
    }

    async fn execute_cmd(&mut self, client: &dyn Client) -> anyhow::Result<(), anyhow::Error> {
        let consumer_groups = client
            .get_consumer_groups(
                &self.get_consumer_groups.stream_id,
                &self.get_consumer_groups.topic_id,
            )
            .await
            .with_context(|| {
                format!(
                    "Problem getting consumer groups for stream with ID: {} and topic with ID: {}",
                    self.get_consumer_groups.stream_id, self.get_consumer_groups.topic_id
                )
            })?;

        match self.output {
            GetConsumerGroupsOutput::Table => {
                let mut table = Table::new();
                table.set_header(vec!["ID", "Name", "Partitions Count", "Members Count"]);
                consumer_groups.iter().for_each(|group| {
                    table.add_row(vec![
                        format!("{}", group.id),
                        group.name.clone(),
                        format!("{}", group.partitions_count),
                        format!("{}", group.members_count),
                    ]);
                });

                event!(target: PRINT_TARGET, Level::INFO, "{table}");
            }
            GetConsumerGroupsOutput::List => {
                consumer_groups.iter().for_each(|group| {
                    event!(target: PRINT_TARGET, Level::INFO,
                        "{}|{}|{}|{}",
                        group.id,
                        group.name,
                        group.partitions_count,
                        group.members_count,
                    );
                });
            }
        }

        Ok(())
    }
}
