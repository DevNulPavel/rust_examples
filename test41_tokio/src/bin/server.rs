#[allow(unused_imports)]

use std::io::Cursor;
use std::process::Output;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::prelude::*;
use tokio::runtime::Handle;
use tokio::process::{ Command, Child };
use tokio::sync::{ Mutex };
use tokio::sync::mpsc::{channel, unbounded_channel, Receiver, Sender, UnboundedReceiver, UnboundedSender};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{ TcpListener, TcpStream};
use tokio::net::tcp::{ Incoming, ReadHalf, WriteHalf };
use futures::stream::StreamExt;
use bytes::{Bytes, BytesMut, Buf, BufMut};
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian, LittleEndian};
use serde::Deserialize;
use test41_tokio::{EmptyResult, StringResult, save_file_from_socket};

type PathSender = UnboundedSender<(PathBuf, ResultSender)>;
type PathReceiver = UnboundedReceiver<(PathBuf, ResultSender)>;
type ResultSender = UnboundedSender<(PathBuf, String)>;
type ResultReceiver = UnboundedReceiver<(PathBuf, String)>;

struct ServerStatus{
    active_converts: u16,
    max_active_converts: u16,
    receive_wait: futures::channel::mpsc::Receiver<bool>
}

#[derive(Deserialize)]
struct ReceivedMeta{
    file_size: usize,
    file_name: String,
}

async fn get_md5(path: &Path) -> StringResult {
    let path_str = match path.to_str(){
        Some(path_str) => {
            path_str
        },
        None => {
            return Err("Empty path".into());
        }
    };

    let child = Command::new("md5")
        .arg(path_str)
        .output();

    let out: Output = child.await?;    

    println!("the command exited with: {:?}", out);

    if out.status.success() {
        let text = String::from_utf8(out.stdout)?;
        Ok(text)
    }else{
        let text = String::from_utf8(out.stderr)?;
        Err(text.into())
    }
}

async fn process_files(mut receiver: PathReceiver)-> EmptyResult{
    // TODO: Ограничение на количество одновременных обработок
    //let (input_sender, input_receiver) = channel::<PathBuf>(8);
    //let (output_sender, output_receiver) = channel::<(PathBuf, String)>(8);
    loop {
        let (received_path, response_ch) = match receiver.recv().await{
            Some(data) => {
                data
            },
            None => {
                break;
            }
        };

        if let Ok(md5_res) = get_md5(&received_path).await {
            response_ch.send((received_path, md5_res))?;
        }
    }

    Ok(())
}

async fn process_sending_data<'a>(mut writer: WriteHalf<'a>, mut receiver: ResultReceiver) {
}


async fn process_incoming_data<'a>(mut reader: ReadHalf<'a>, process_sender: PathSender, sock_channel: ResultSender) -> EmptyResult {
    loop {
        let json_size: usize = reader.read_u16().await? as usize;

        let mut buffer: BytesMut = BytesMut::with_capacity(json_size);
        reader.read_exact(&mut buffer).await?;

        let json: ReceivedMeta = serde_json::from_slice(&buffer).unwrap();

        let file_path = PathBuf::new()
            .join(json.file_name);
        let save_result = save_file_from_socket(&mut reader, json.file_size, &file_path).await;
        if save_result.is_ok() {
            process_sender.send((file_path, sock_channel.clone()))?;
        }
    }
}

async fn process_connection(mut sock: TcpStream, process_sender: PathSender){
    //let sock = std::sync::Arc::from(sock);
    // Получаем отдельные каналы сокета на чтение и на запись
    let (reader, writer) = sock.split();
    let reader: ReadHalf = reader;
    let writer: WriteHalf = writer;

    let (sender, receiver) = unbounded_channel::<(PathBuf, String)>();
    
    let reader_join = process_incoming_data(reader, process_sender, sender);
    let writer_join = process_sending_data(writer, receiver);

    let _ = reader_join.await;
    let _ = writer_join.await;
    // tokio::join!(reader_join, writer_join);

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

fn create_processing() -> (impl futures::Future<Output = EmptyResult>, PathSender) {
    let (processing_sender, input_receiver) = unbounded_channel::<(PathBuf, ResultSender)>();
    
    let process_file_future = process_files(input_receiver);

    (process_file_future, processing_sender)
}

#[tokio::main]
async fn main() {
    let (process_file_future, processing_sender) = create_processing();

    let addr = "127.0.0.1:6142";

    // Дожидаемся успешного создания серверного сокета
    let mut listener: TcpListener = TcpListener::bind(addr)
        .await
        .unwrap();
    
    // TODO: !!! убрать, либо чистить с проверкой результата
    let mut active_futures = vec![];

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
                    let sender = processing_sender.clone();
                    let fut = process_connection(sock, sender);
                    active_futures.push(fut);

                    // Не работает, так как фьючи могут пережить каналы и обработку данных
                    //tokio::spawn(fut);
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

    process_file_future.await;

    // TODO: завершение по Ctrl + C
}