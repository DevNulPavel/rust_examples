use std::{
    error::{
        Error
    }
};
use async_trait::{
    async_trait
};
use log::{
    error,
    info
};
use crate::{
    uploaders::{
        UploadResultData
    }
};
use super::{
    ResultSender
};

pub struct TerminalSender{
}
#[async_trait(?Send)]
impl ResultSender for TerminalSender {
    async fn send_result(&self, result: &UploadResultData){
        info!("Uploading task success: {}", result);
    }
    async fn send_error(&self, err: &dyn Error){
        error!("Uploading task error: {}", err);
    }
}