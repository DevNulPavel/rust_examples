////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, thiserror::Error)]
pub enum P2PError {
    /// Ошибка шифрования трафика
    #[error("encryption error -> {0}")]
    Encryption(#[from] libp2p::noise::Error),
}
