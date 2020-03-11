
// Тип используемых ошибок в коде
#[derive(Debug)]
pub enum MessageError{
    EmptyEmail,
    EmptyUser,
    IsNotFound,
    UsersListIsMissing,
    ChannelDidNotOpen,
    QRFileDidNotCreate(qrcode::types::QrError),
    QRFileDidNotConvertToPng(std::io::Error),
    QRFileDidNotOpen(std::io::Error),
    RequestError(reqwest::Error) // Обертка над ошибкой запроса
}

// Пустая автоматическая реализация
impl std::error::Error for MessageError{
}

// Код для автоматической конвертации ошибки в коде
impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Код для автоматической конвертации ошибки в коде
impl From<reqwest::Error> for MessageError {
    fn from(err: reqwest::Error) -> Self {
        Self::RequestError(err)
    }
}

impl From<qrcode::types::QrError> for MessageError {
    fn from(err: qrcode::types::QrError) -> Self {
        Self::QRFileDidNotCreate(err)
    }
}
