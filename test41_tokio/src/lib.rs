
// TODO: ???
// Описание модулей используются только в main.rs, lib.rs, mod.rs
// В других файлах надо использовать:
//  - use self:: - для получения чего-то из текущего модуля (например, подмодуля у того же файла), аналог "./"
//  - use super:: - для получения чего-то из родительского модуля, включая приватные элементы, аналог "../"
//  - use crate:: - импорт чего-то от корня крейта, аналог "/"
//
// Модули – не то же самое, что файлы, но существует естественная аналогия между модулями и файлами и каталогами в
// файловой системе Unix. Ключевое слово use создает псевдонимы точно так же, как команда ln создает ссылки.
// Пути, как и имена файлов, могут быть абсолютными и относительными.
// Ключевые слова self и super – аналоги специальных каталогов . и ..
// А extern crate включает в проект корневой модуль еще одного крейта – почти то же, что монтирование файловой системы.
//
// Папку с корневым файлом mod.rs надо воспринимать как модуль с подмодулями в виде других файликов
// другие файлы .rs лучше воспринимать как папку с корневым файлом mod.rs
// https://users.rust-lang.org/t/importing-module-from-another-module/18172/9


use std::io::Cursor;
use std::string::String;
use std::path::Path;
use tokio::prelude::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::runtime::Handle;
use tokio::net::{ TcpListener, TcpStream};
use tokio::net::tcp::{ Incoming, ReadHalf, WriteHalf };
use futures::stream::StreamExt;
use bytes::{Bytes, BytesMut, Buf, BufMut};
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian, LittleEndian};
use serde::{Deserialize, Serialize};

pub type EmptyResult = Result<(), Box<dyn std::error::Error>>;
pub type StringResult = Result<String, Box<dyn std::error::Error>>;

#[derive(Deserialize, Serialize)]
pub struct FileMeta{
    pub file_size: usize,
    pub file_name: String,
}

pub async fn write_file_to_socket<'a>(writer: &mut WriteHalf<'a>, path: &Path) -> EmptyResult {
    // Открываем асинхронный файлик
    let mut file: tokio::fs::File = tokio::fs::File::open(path)
        .await?;
    
    let file_meta = file.metadata()
        .await?;
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
        return Err(format!("Read from socet failed: {:?}", path).into());
    }
}