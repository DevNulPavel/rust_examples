////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, thiserror::Error)]
pub(crate) enum BlockOnError {
    #[error("join error: {0}")]
    Join(#[from] super::handle::JoinError),

    #[error("already locked")]
    AlreadyBlocked,
}
