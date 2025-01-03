use crate::error::P2PError;
use libp2p::{
    core::upgrade::Version, core::Transport as TransportTrait, identity::Keypair, noise,
    swarm::ConnectionHandler, tcp::tokio::Transport as TokioTransport,
};
use std::{error::Error, time::Duration};

////////////////////////////////////////////////////////////////////////////////

/// Создание транспорта для подключения
pub(super) fn create_transport(
    local_key: &Keypair,
) -> Result<impl TransportTrait<Output = impl ConnectionHandler, Error = impl Error>, P2PError> {
    // Создаем конфиг для шифрования сначала с использованием того же алгоритма
    // 25519, который был уже использован для создание пар ключей
    let noise_config = noise::Config::new(local_key)?;

    // Различные настройки работы TCP
    let tcp_config = libp2p::tcp::Config::new().nodelay(true);

    // TODO: Настройка мультиплексирования
    let transport = TokioTransport::new(tcp_config)
        .upgrade(Version::V1)
        .authenticate(noise_config)
        .multiplex(todo!())
        .timeout(Duration::from_secs(60))
        .boxed();

    Ok(transport)

    // let noise_keys = noise::Keypair::<noise::X25519Spec>::default()
    //     .into_authentic(local_key)
    //     .expect("Signing libp2p-noise static DH keypair failed.");

    // TokioTcpConfig::new()
    //     .nodelay(true)
    //     .upgrade(upgrade::Version::V1)
    //     .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
    //     .multiplex(mplex::MplexConfig::new())
    //     .boxed()

    // todo!()
}
