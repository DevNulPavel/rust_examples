use actix_web::{
    client::{
        SendRequestError,
        JsonPayloadError
    }
};
use super::{
    view_open_response::{
        ViewOpenErrorInfo,
        ViewUpdateErrorInfo,
    }
};

#[derive(Debug)]
pub enum SlackViewError{
    RequestErr(SendRequestError),
    JsonParseError(JsonPayloadError),
    OpenError(ViewOpenErrorInfo),
    UpdateError(ViewUpdateErrorInfo)
}

impl From<SendRequestError> for SlackViewError {
    fn from(err: SendRequestError) -> SlackViewError {
        SlackViewError::RequestErr(err)
    }
}

impl From<JsonPayloadError> for SlackViewError {
    fn from(err: JsonPayloadError) -> SlackViewError {
        SlackViewError::JsonParseError(err)
    }
}

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