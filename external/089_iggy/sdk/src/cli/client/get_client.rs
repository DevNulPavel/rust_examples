use crate::cli_command::{CliCommand, PRINT_TARGET};
use crate::client::Client;
use crate::system::get_client::GetClient;
use anyhow::Context;
use async_trait::async_trait;
use comfy_table::{presets::ASCII_NO_BORDERS, Table};
use tracing::{event, Level};

pub struct GetClientCmd {
    get_client: GetClient,
}

impl GetClientCmd {
    pub fn new(client_id: u32) -> Self {
        Self {
            get_client: GetClient { client_id },
        }
    }
}

#[async_trait]
impl CliCommand for GetClientCmd {
    fn explain(&self) -> String {
        format!("get client with ID: {}", self.get_client.client_id)
    }

    async fn execute_cmd(&mut self, client: &dyn Client) -> anyhow::Result<(), anyhow::Error> {
        let client_details = client
            .get_client(self.get_client.client_id)
            .await
            .with_context(|| {
                format!(
                    "Problem getting client with ID: {}",
                    self.get_client.client_id
                )
            })?;

        if client_details.is_none() {
            event!(target: PRINT_TARGET, Level::INFO, "Client with ID: {} was not found", self.get_client.client_id);
            return Ok(());
        }

        let client_details = client_details.unwrap();
        let mut table = Table::new();

        table.set_header(vec!["Property", "Value"]);
        table.add_row(vec![
            "Client ID",
            format!("{}", client_details.client_id).as_str(),
        ]);

        let user = match client_details.user_id {
            Some(user_id) => format!("{}", user_id),
            None => String::from("None"),
        };
        table.add_row(vec!["User ID", user.as_str()]);

        table.add_row(vec!["Address", client_details.address.as_str()]);
        table.add_row(vec!["Transport", client_details.transport.as_str()]);
        table.add_row(vec![
            "Consumer Groups Count",
            format!("{}", client_details.consumer_groups_count).as_str(),
        ]);

        if client_details.consumer_groups_count > 0 {
            let mut consumer_groups = Table::new();
            consumer_groups.load_preset(ASCII_NO_BORDERS);
            consumer_groups.set_header(vec!["Stream ID", "Topic ID", "Consumer Group ID"]);
            for consumer_group in client_details.consumer_groups {
                consumer_groups.add_row(vec![
                    format!("{}", consumer_group.stream_id).as_str(),
                    format!("{}", consumer_group.topic_id).as_str(),
                    format!("{}", consumer_group.group_id).as_str(),
                ]);
            }

            table.add_row(vec![
                "Consumer Groups Details",
                consumer_groups.to_string().as_str(),
            ]);
        }

        event!(target: PRINT_TARGET, Level::INFO, "{table}");

        Ok(())
    }
}
