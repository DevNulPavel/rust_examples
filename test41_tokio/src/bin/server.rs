#[allow(unused_imports)]

use std::io::Cursor;
use tokio::prelude::*;
use tokio::runtime::Handle;
use tokio::net::{ TcpListener, TcpStream};
use tokio::net::tcp::{ Incoming, ReadHalf, WriteHalf };
use futures::stream::StreamExt;
use bytes::{Bytes, BytesMut, Buf, BufMut};
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian, LittleEndian};
use serde::Deserialize;

struct ServerStatus{
    active_converts: u16,
    max_active_converts: u16,
    receive_wait: futures::channel::mpsc::Receiver<bool>
}

#[derive(Deserialize)]
struct ReceivedMeta{
    file_size: usize,
    filename: String,
}

async fn process_sending_data<'a>(mut writer: WriteHalf<'a>) {
}

async fn save_file_from_socket<'a>(reader: &mut ReadHalf<'a>, info: ReceivedMeta){
    let mut file: tokio::fs::File = tokio::fs::File::create(info.filename)
        .await
        .unwrap();
    
    let mut data_buffer: [u8; 1024] = [0; 1024];
    let mut size_left = info.file_size;

    while size_left > 0 {
        let read_size = data_buffer.len().min(size_left);
        let buffer_slice = &mut data_buffer[0..read_size];
        let read_result = reader.read_exact(buffer_slice).await;
        match read_result {
            Ok(size) => {
                let result_slice = &mut data_buffer[0..size];
                file.write(result_slice)
                    .await
                    .unwrap();
                size_left -= size;
            },
            Err(e) => {
                eprintln!("Read from socket error: {}", e);
                break;
            }
        }
    }
}

async fn process_incoming_data<'a>(mut reader: ReadHalf<'a>) {
    loop {
        let mut data_size_buffer: [u8; 4] = [0; 4];
        let read_result = reader.read_exact(&mut data_size_buffer).await;
        match read_result {
            Ok(size) if size == data_size_buffer.len() => {
                let json_size = u32::from_le_bytes(data_size_buffer) as usize;
                let mut buffer: BytesMut = BytesMut::with_capacity(json_size);
                match reader.read_exact(&mut buffer).await {
                    Ok(_) => {
                        let json: ReceivedMeta = serde_json::from_slice(&buffer).unwrap();
                        save_file_from_socket(&mut reader, json).await;
                    },
                    Err(err) => {
                        eprintln!("Read from socket error: {}", err);
                        break;
                    }
                }
            },
            Ok(size) => {
                eprintln!("Read from socket: size == {}", size);
                break;
            },
            Err(err) => {
                eprintln!("Read from socket error: {}", err);
                break;
            }
        }
    }
}

async fn process_connection(mut sock: TcpStream){
    //let sock = std::sync::Arc::from(sock);
    // Получаем отдельные каналы сокета на чтение и на запись
    let (reader, writer) = sock.split();
    let reader: ReadHalf = reader;
    let writer: WriteHalf = writer;
    
    let reader_join = process_incoming_data(reader);
    let writer_join = process_sending_data(writer);

    tokio::join!(reader_join, writer_join);

    /*let copy_result = tokio::io::copy(&mut reader, &mut writer).await;
    match copy_result {
        Ok(amt) => {
            println!("wrote {} bytes", amt);
        }
        Err(err) => {
            eprintln!("IO error {:?}", err);
        }
    }*/
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:6142";

    // Дожидаемся успешного создания серверного сокета
    let mut listener: TcpListener = TcpListener::bind(addr)
        .await
        .unwrap();
    
    // Создаем обработчик
    let server = async move {
        // Получаем поток новых соединений
        let mut incoming: Incoming = listener.incoming();

        // Получаем кавые соединения каждый ра
        while let Some(conn) = incoming.next().await {
            match conn {
                Ok(sock) => {
                    // Закидываем задачу по работе с нашим сокетом в отдельную функцию
                    // tokio::spawn - неблокирующая функция, закидывающая задачу в обработку
                    tokio::spawn(process_connection(sock));
                }
                Err(e) => {
                    eprintln!("accept failed = {:?}", e)
                },
            }
        }
    };
    
    println!("Server running on localhost:6142");
    
    // Блокируемся на выполнении сервера
    server.await;

    // TODO: завершение по Ctrl + C
}