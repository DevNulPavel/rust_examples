use std::{
    error::{
        Error
    },
    sync::{
        Arc
    },
    pin::{
        Pin
    }
};
use async_trait::{
    async_trait
};
use crate::{
    uploaders::{
        UploadResult
    }
};

#[async_trait(?Send)]
pub trait ResultSender {
    async fn send_result(&self, result: &UploadResult);
    async fn send_error(&self, err: &dyn Error);
}