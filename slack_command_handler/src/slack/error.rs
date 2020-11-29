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
pub enum SlackViewError{
    RequestErr(Error),
    JsonParseError(Error),
    OpenError(ViewOpenErrorInfo),
    UpdateError(ViewUpdateErrorInfo)
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
impl From<ViewOpenErrorInfo> for SlackViewError {
    fn from(err: ViewOpenErrorInfo) -> SlackViewError {
        SlackViewError::OpenError(err)
    }
}
impl From<ViewUpdateErrorInfo> for SlackViewError {
    fn from(err: ViewUpdateErrorInfo) -> SlackViewError {
        SlackViewError::UpdateError(err)
    }
}