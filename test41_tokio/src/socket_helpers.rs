use std::string::String;
use std::path::Path;
use tokio::io::{ AsyncReadExt, AsyncWriteExt };
use tokio::net::tcp::{ ReadHalf, WriteHalf };
use serde::{Deserialize, Serialize};
use crate::types::EmptyResult;

#[derive(Deserialize, Serialize, Debug)]
pub struct FileMeta{
    pub file_size: usize,
    pub file_name: String,
}

pub async fn write_file_to_socket<'a>(writer: &mut WriteHalf<'a>, path: &Path) -> EmptyResult {
    // Открываем асинхронный файлик
    let mut file: tokio::fs::File = tokio::fs::File::open(path).await?;
    
    let file_meta = file.metadata().await?;
    let file_size = file_meta.len() as usize;

    let meta = FileMeta{
        file_size,
        file_name: path.to_str().unwrap().to_owned()
    };
    let meta = serde_json::to_vec(&meta).unwrap();

    writer.write_u16(meta.len() as u16).await?;
    writer.write_all(&meta).await?;

    tokio::io::copy(&mut file, writer).await?;

    Ok(())
}

pub async fn save_file_from_socket<'a>(reader: &mut ReadHalf<'a>, file_size: usize, path: &Path) -> EmptyResult {
    if file_size == 0{
        panic!("File size can't be zero: {:?}", path);
    }

    // Создаем папку
    if !path.exists(){
        if let Some(folder) = path.parent(){
            if !folder.exists(){
                tokio::fs::create_dir_all(folder).await?;   
            }
        }
    }
    // Создаем асинхронно файлик
    let mut file: tokio::fs::File = tokio::fs::File::create(path)
        .await?;
    
    let mut data_buffer: [u8; 1024] = [0; 1024];
    let mut size_left = file_size;

    while size_left > 0 {
        // Сколько чатаем из сокета
        let read_size = data_buffer.len().min(size_left);
        
        // Слайс нужного размера
        let buffer_slice = &mut data_buffer[0..read_size];

        // Читаем из файлика
        let read_size_result = match reader.read_exact(buffer_slice).await {
            Ok(read_size) => {
                read_size
            },
            Err(err) => {
                tokio::fs::remove_file(path).await?;
                return Err(err.into());
            }   
        };

        // Проверяем, что прочитали норм все
        if read_size_result == read_size {
            // Слайс прочитанных данных
            let result_slice = &mut data_buffer[0..read_size_result];

            // Пишем в файлик
            if let Err(e) = file.write_all(result_slice).await {
                tokio::fs::remove_file(path).await?;
                return Err(e.into());
            }
            
            // Уменьшаем оставшееся количество байт на чтение
            size_left -= read_size_result;
        }else{
            break;
        }
    }

    if size_left == 0 {
        Ok(())
    }else{
        //format!("Read from socet failed: {:?}", path).into()
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "").into());
    }
}