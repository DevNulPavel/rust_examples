use crate::streaming::clients::client_manager::{Client, Transport};
use crate::streaming::session::Session;
use crate::streaming::systems::system::System;
use iggy::error::IggyError;
use iggy::identifier::Identifier;
use iggy::locking::IggySharedMut;
use iggy::locking::IggySharedMutFn;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info};

impl System {
    pub async fn add_client(&self, address: &SocketAddr, transport: Transport) -> Arc<Session> {
        let mut client_manager = self.client_manager.write().await;
        let session = client_manager.add_client(address, transport);
        info!("Added {transport} client with session: {session} for IP address: {address}");
        self.metrics.increment_clients(1);
        session
    }

    pub async fn delete_client(&self, client_id: u32) {
        let consumer_groups: Vec<(u32, u32, u32)>;

        {
            let mut client_manager = self.client_manager.write().await;
            let client = client_manager.delete_client(client_id).await;
            if client.is_none() {
                error!("Client with ID: {client_id} was not found in the client manager.",);
                return;
            }

            self.metrics.decrement_clients(1);
            let client = client.unwrap();
            let client = client.read().await;
            consumer_groups = client
                .consumer_groups
                .iter()
                .map(|c| (c.stream_id, c.topic_id, c.group_id))
                .collect();

            info!(
                "Deleted {} client with ID: {} for IP address: {}",
                client.transport, client.session.client_id, client.session.ip_address
            );
        }

        for (stream_id, topic_id, consumer_group_id) in consumer_groups.into_iter() {
            _ = self
                .leave_consumer_group_by_client(
                    &Identifier::numeric(stream_id).unwrap(),
                    &Identifier::numeric(topic_id).unwrap(),
                    &Identifier::numeric(consumer_group_id).unwrap(),
                    client_id,
                )
                .await
        }
    }

    pub async fn get_client(
        &self,
        session: &Session,
        client_id: u32,
    ) -> Result<IggySharedMut<Client>, IggyError> {
        self.ensure_authenticated(session)?;
        self.permissioner.get_client(session.get_user_id())?;
        let client_manager = self.client_manager.read().await;
        client_manager.get_client(client_id)
    }

    pub async fn get_clients(
        &self,
        session: &Session,
    ) -> Result<Vec<IggySharedMut<Client>>, IggyError> {
        self.ensure_authenticated(session)?;
        self.permissioner.get_clients(session.get_user_id())?;
        let client_manager = self.client_manager.read().await;
        Ok(client_manager.get_clients())
    }
}
