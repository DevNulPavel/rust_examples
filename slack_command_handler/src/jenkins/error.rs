use std::{
    str::{
        Utf8Error
    }
};
use reqwest::{
    Error
};
use serde_xml_rs::{
    Error as XMLError
};

#[derive(Debug)]
pub enum JenkinsError{
    RequestError(Error, String),
    BodyParseError(Error, String),
    JsonParseError(Error, String),
    XMLParseError(XMLError),
    ResponseUtf8ConvertError(Utf8Error),
    LogicalError(String)
}

// impl From<SendRequestError> for JenkinsError {
//     fn from(err: SendRequestError) -> JenkinsError {
//         JenkinsError::RequestError(err)
//     }
// }
// impl From<PayloadError> for JenkinsError {
//     fn from(err: PayloadError) -> JenkinsError {
//         JenkinsError::BodyParseError(err)
//     }
// }
impl From<XMLError> for JenkinsError {
    fn from(err: XMLError) -> JenkinsError {
        JenkinsError::XMLParseError(err)
    }
}
// impl From<JsonPayloadError> for JenkinsError {
//     fn from(err: JsonPayloadError) -> JenkinsError {
//         JenkinsError::JsonParseError(err)
//     }
// }
impl From<Utf8Error> for JenkinsError {
    fn from(err: Utf8Error) -> JenkinsError {
        JenkinsError::ResponseUtf8ConvertError(err)
    }
}