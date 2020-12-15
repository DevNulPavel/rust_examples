#[derive(Debug)]
pub enum HabrError {
    RequestError(reqwest::Error),
}

impl From<reqwest::Error> for HabrError {
    fn from(err: reqwest::Error) -> HabrError {
        HabrError::RequestError(err)
    }
}
