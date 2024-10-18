use super::server_command::ServerCommand;
use crate::configs::server::ServerConfig;
use crate::streaming::systems::system::SharedSystem;

pub struct ServerCommandHandler<'a> {
    system: SharedSystem,
    config: &'a ServerConfig,
}

impl<'a> ServerCommandHandler<'a> {
    pub fn new(system: SharedSystem, config: &'a ServerConfig) -> Self {
        Self { system, config }
    }

    pub fn install_handler<C, E>(&mut self, mut executor: E) -> Self
    where
        E: ServerCommand<C> + Send + Sync + 'static,
    {
        let (sender, receiver) = flume::unbounded();
        let system = self.system.clone();
        executor.start_command_sender(system.clone(), self.config, sender);
        executor.start_command_consumer(system.clone(), self.config, receiver);
        Self {
            system,
            config: self.config,
        }
    }
}
