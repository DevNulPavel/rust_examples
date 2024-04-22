use async_std::{
    sync::{
        Sender
    }
};
use crate::{
    utils::{
        FromSlice
    }
};
use super::{
    writer::{
        WriterUserCommand
    }
};

#[derive(Debug)]
pub struct UtpStream {
    // reader_command: Sender<ReaderCommand>,
    // reader_result: Receiver<ReaderResult>,
    
    /// Канал для отправки полезных данных
    pub writer_user_command: Sender<WriterUserCommand>,
}

impl UtpStream {
    pub async fn read(&self, _data: &mut [u8]) {
        // self.reader_command.send(ReaderCommand {
        //     length: data.len()
        // }).await;

        // self.reader_result.recv().await;
    }

    /// Отправка в канал пользовательской комманды с данными
    pub async fn write(&self, data: &[u8]) {
        let data = Vec::from_slice(data);
        self.writer_user_command.send(WriterUserCommand { data }).await;
    }
}