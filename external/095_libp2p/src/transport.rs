use crate::error::P2PError;
use libp2p::{
    core::{muxing::StreamMuxerBox, transport, upgrade::Version, Transport as TransportTrait},
    identity::Keypair,
    noise,
    tcp::tokio::Transport as TokioTransport,
    PeerId,
};

////////////////////////////////////////////////////////////////////////////////

/// Создание транспорта для подключения
pub(super) fn create_transport(
    local_key: &Keypair,
) -> Result<transport::Boxed<(PeerId, StreamMuxerBox)>, P2PError> {
    // Создаем конфиг для шифрования сначала с использованием того же алгоритма
    // 25519, который был уже использован для создание пар ключей
    let noise_config = noise::Config::new(local_key)?;

    // Различные настройки работы TCP
    let tcp_config = libp2p::tcp::Config::new().nodelay(true);

    // Конфиг мультиплексирования
    let multiplex_config = libp2p::yamux::Config::default();

    // Создаем в общий транспорт
    let transport = TokioTransport::new(tcp_config)
        .upgrade(Version::V1)
        .authenticate(noise_config)
        .multiplex(multiplex_config)
        .boxed();

    Ok(transport)
}
