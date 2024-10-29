////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, thiserror::Error)]
pub enum BlockOnError {
    #[error("join error -> {0}")]
    Join(#[from] JoinError),

    #[error("already locked")]
    AlreadyBlocked,
}

////////////////////////////////////////////////////////////////////////////////

/// Ошибка ожидания завершения работы
#[derive(Debug, thiserror::Error)]
#[error("join fail -> result channel dropped")]
pub struct JoinError;
