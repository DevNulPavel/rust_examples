#[derive(Debug)]
pub enum AppError {
    RequestError(reqwest::Error),
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> AppError {
        AppError::RequestError(err)
    }
}
