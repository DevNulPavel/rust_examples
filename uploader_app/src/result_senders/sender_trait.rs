use std::{
    error::{
        Error
    }
};
use async_trait::{
    async_trait
};
use crate::{
    uploaders::{
        UploadResultData
    }
};

#[async_trait(?Send)]
pub trait ResultSender {
    async fn send_result(&self, result: &UploadResultData);
    async fn send_error(&self, err: &dyn Error);
}