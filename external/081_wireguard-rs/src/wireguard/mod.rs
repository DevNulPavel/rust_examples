/// Подмодуль Wireguard представляет собой полную и чистую Rust реализацию
/// 
/// Устройство описанное здесь не зависит от определенной IO реализации или UAPI
/// и может быть создано в тестах с заглушкой IO реализации.
/// 
/// Код на данном уровне служит клеем машины состояний для Handshake
/// и крипто-роутера, таким образом каждый Wireguard пир состоит из одного
/// пира хендшейка + одного пира роутера.
mod constants;
mod handshake;
mod peer;
mod queue;
mod router;
mod timers;
mod types;
mod workers;

#[cfg(test)]
mod tests;

#[allow(clippy::module_inception)]
mod wireguard;

// represents a WireGuard interface
pub use wireguard::WireGuard;

#[cfg(test)]
use super::platform::dummy;

use super::platform::{tun, udp, Endpoint};
use types::KeyPair;
