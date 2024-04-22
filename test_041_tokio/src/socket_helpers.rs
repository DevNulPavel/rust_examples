use std::{
    string::{
        String
    }
};
use std::{
    path::{
        Path
    }
};
use tokio::{
    io::{ 
        AsyncReadExt, 
        AsyncWriteExt 
    },
    fs::{
        File
    },
    net::{
        tcp::{ 
            ReadHalf, 
            WriteHalf 
        }
    }
};
use serde::{
    Deserialize, 
    Serialize
};
use crate::{
    types::{
        EmptyResult
    }
};

#[derive(Deserialize, Serialize, Debug)]
pub struct FileMeta{
    pub file_size: usize,
    pub file_name: String,
}

/// Записываем файлик в сокет
pub async fn write_file_to_socket<'a>(tcp_writer: &mut WriteHalf<'a>, 
                                      path: &Path) -> EmptyResult {
    // Открываем асинхронный файлик
    let mut file: File = File::open(path)
        .await?;
    
    let file_meta = file
        .metadata()
        .await?;
    let file_size = file_meta.len() as usize;

    let meta = FileMeta{
        file_size,
        file_name: path.to_str().unwrap().to_owned()
    };
    let meta = serde_json::to_vec(&meta).unwrap();

    // Пишем размер меты в сокет
    tcp_writer.write_u16(meta.len() as u16).await?;
    // Пишем в сокет мету
    tcp_writer.write_all(&meta).await?;

    // Копируем непосредственно данные
    tokio::io::copy(&mut file, tcp_writer).await?;

    Ok(())
}

/// Пишем файлик из сокета на диск
pub async fn save_file_from_socket<'a>(reader: &mut ReadHalf<'a>, 
                                       file_size: usize, 
                                       file_path: &Path) -> EmptyResult {
    if file_size == 0{
        panic!("File size can't be zero: {:?}", file_path);
    }

    // Создаем папку
    if !file_path.exists(){
        if let Some(folder) = file_path.parent(){
            if !folder.exists(){
                tokio::fs::create_dir_all(folder).await?;   
            }
        }
    }

    // Создаем асинхронно файлик
    let mut file: File = File::create(file_path)
        .await?;
    
    // Создаем буффер для временных данных
    let mut data_buffer: [u8; 1024] = [0; 1024];
    let mut size_left = file_size;

    while size_left > 0 {
        // Сколько чатаем из сокета
        let read_size = data_buffer.len().min(size_left);
        
        // Слайс нужного размера
        let buffer_slice = &mut data_buffer[0..read_size];

        // Читаем из файлика точное количество байт
        let read_size_result = match reader.read_exact(buffer_slice).await {
            Ok(read_size) => {
                read_size
            },
            Err(err) => {
                // Если произошла ошибка, то и файлик завершаем
                tokio::fs::remove_file(file_path).await?;
                return Err(err.into());
            }   
        };

        // Проверяем, что прочитали норм все
        if read_size_result == read_size {
            // Слайс прочитанных данных
            let result_slice = &mut data_buffer[0..read_size_result];

            // Пишем в файлик, либо удаляем если ошибка
            if let Err(e) = file.write_all(result_slice).await {
                tokio::fs::remove_file(file_path).await?;
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