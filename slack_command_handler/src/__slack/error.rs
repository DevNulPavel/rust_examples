use std::{
    collections::{
        HashMap
    }
};
use reqwest::{
    Error
};
use serde::{
    Deserialize
};
use serde_json::{
    Value
};


#[derive(Deserialize, Debug)]
pub struct ViewOpenErrorInfo{
    error: String,
    response_metadata: HashMap<String, Value>
}

#[derive(Deserialize, Debug)]
pub struct ViewUpdateErrorInfo{
    error: String
}

#[derive(Debug)]
pub enum SlackError{
    RequestErr(Error),
    JsonParseError(Error),
    ViewOpenError(ViewOpenErrorInfo),
    UpdateError(ViewUpdateErrorInfo),
    Custom(String),
}

// impl From<SendRequestError> for SlackViewError {
//     fn from(err: SendRequestError) -> SlackViewError {
//         SlackViewError::RequestErr(err)
//     }
// }
// impl From<JsonPayloadError> for SlackViewError {
//     fn from(err: JsonPayloadError) -> SlackViewError {
//         SlackViewError::JsonParseError(err)
//     }
// }
impl From<ViewOpenErrorInfo> for SlackError {
    fn from(err: ViewOpenErrorInfo) -> SlackError {
        SlackError::ViewOpenError(err)
    }
}
impl From<ViewUpdateErrorInfo> for SlackError {
    fn from(err: ViewUpdateErrorInfo) -> SlackError {
        SlackError::UpdateError(err)
    }
}