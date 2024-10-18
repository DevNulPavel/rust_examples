use crate::binary::mapper;
use crate::binary::sender::Sender;
use crate::streaming::session::Session;
use crate::streaming::systems::system::SharedSystem;
use iggy::error::IggyError;
use iggy::locking::IggySharedMutFn;
use iggy::system::get_client::GetClient;
use tracing::debug;

pub async fn handle(
    command: GetClient,
    sender: &mut dyn Sender,
    session: &Session,
    system: &SharedSystem,
) -> Result<(), IggyError> {
    debug!("session: {session}, command: {command}");
    let bytes;
    {
        let system = system.read().await;
        let client = system.get_client(session, command.client_id).await;
        if client.is_err() {
            sender.send_empty_ok_response().await?;
            return Ok(());
        }

        {
            let client = client?;
            let client = client.read().await;
            bytes = mapper::map_client(&client);
        }
    }
    sender.send_ok_response(&bytes).await?;
    Ok(())
}
