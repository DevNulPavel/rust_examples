#[allow(unused_imports)]

use std::io::Cursor;
use std::process::Output;
use std::path::{Path, PathBuf};
use std::sync::Arc;
// use tokio::prelude::*;
use tokio::process::{ Command };
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{ TcpListener, TcpStream};
use tokio::net::tcp::{ Incoming, ReadHalf, WriteHalf };
use futures::stream::StreamExt;
use bytes::BytesMut;
use test41_tokio::*;

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
        println!("Text return: {}", text);
        Ok(text)
    }else{
        let text = String::from_utf8(out.stderr)?;
        println!("Error return: {}", text);
        Err(text.into())
    }
}

async fn process_files(mut receiver: PathReceiver)-> EmptyResult {
    // TODO: Ограничение на количество одновременных обработок
    //let (input_sender, input_receiver) = channel::<PathBuf>(8);
    //let (output_sender, output_receiver) = channel::<(PathBuf, String)>(8);

    let semaphore = tokio::sync::Semaphore::new(8);
    loop {
        let (received_path, response_ch) = match receiver.recv().await{
            Some(comand) => {
                match comand {
                    ProcessCommand::Process(data) => { 
                        data
                    },
                    ProcessCommand::Stop => {
                        break;
                    }
                }
            },
            None => {
                break;
            }
        };

        let permit = semaphore.acquire().await;

        tokio::time::delay_for(std::time::Duration::from_millis(4000)).await;

        if let Ok(md5_res) = get_md5(&received_path).await {
            println!("Send to channel: {}", md5_res);
            drop(permit);
            response_ch.send((received_path, md5_res))?;
            println!("Send to channel success");
        }
    }

    println!("Processing exit");
    Ok(())
}

async fn process_sending_data<'a>(mut writer: WriteHalf<'a>, 
                                  mut receiver: ResultReceiver, 
                                  mut stop_receiver: broadcast::Receiver<()>) -> EmptyResult {
    loop {
        let received: ParseResult = tokio::select! {
            received = receiver.recv() => {
                let data = match received {
                    Some(data) => {
                        data
                    },
                    None => {
                        return Err("Channel closed".into());
                    }
                };
                data
            }
            _ = stop_receiver.recv() => {
                return Ok(());
            }
        };

        println!("Data for send: {:?}", received);

        let data: String = received.1;
        writer.write_all(data.as_bytes()).await?;
    }
}

async fn process_incoming_data<'a>(mut reader: ReadHalf<'a>, process_sender: PathSender, 
                                   sock_channel: ResultSender, 
                                   mut stop_receiver: broadcast::Receiver<()>) -> EmptyResult {
    loop {
        let json_size: usize = tokio::select! {
            json_size = reader.read_u16() => {
                json_size? as usize
            }
            _ = stop_receiver.recv() =>{
                return Ok(());
            }
        };

        println!("Received json size: {}", json_size);

        let mut buffer: BytesMut = BytesMut::with_capacity(json_size);
        buffer.resize(json_size, 0);
        reader.read_exact(&mut buffer).await?;

        println!("Received data: {:?}", buffer);

        let json: FileMeta = serde_json::from_slice(&buffer).unwrap();

        println!("Received json: {:?}", json);

        let file_path = PathBuf::new()
            .join(json.file_name);
        let save_result = save_file_from_socket(&mut reader, json.file_size, &file_path).await;
        if save_result.is_ok() {
            let command = ProcessCommand::Process((file_path, sock_channel.clone()));
            process_sender.send(command)?;
        }
    }
}

async fn process_connection(mut sock: TcpStream, process_sender: PathSender, 
                            stop_read_receiver: broadcast::Receiver<()>, 
                            stop_write_receiver: broadcast::Receiver<()>){
    println!("Process connection: {:?}", sock);

    //let sock = std::sync::Arc::from(sock);
    // Получаем отдельные каналы сокета на чтение и на запись
    let (reader, writer) = sock.split();
    let reader: ReadHalf = reader;
    let writer: WriteHalf = writer;

    let (sender, receiver) = unbounded_channel::<(PathBuf, String)>();

    let reader_join = process_incoming_data(reader, process_sender, 
                                                       sender, stop_read_receiver);
    let writer_join = process_sending_data(writer, receiver, stop_write_receiver);

    let (reader_res, writer_res) = tokio::join!(reader_join, writer_join);
    if let Err(reader_res) = reader_res {
        eprintln!("{:?}", reader_res);
    }

    if let Err(writer_res) = writer_res {
        eprintln!("{:?}", writer_res);
    }
}

fn create_processing() -> (impl futures::Future<Output = EmptyResult>, PathSender) {
    let (processing_sender, input_receiver) = unbounded_channel::<ProcessCommand>();
    
    let process_file_future = process_files(input_receiver);

    (process_file_future, processing_sender)
}

#[tokio::main]
async fn main() {
    let (stop_listen_sender, mut stop_listen_receiver) = tokio::sync::broadcast::channel::<()>(1);
    let (stop_read_sender, _) = tokio::sync::broadcast::channel::<()>(1);
    let (stop_write_sender, _) = tokio::sync::broadcast::channel::<()>(1);
    
    let (process_file_future, processing_sender) = create_processing();
    let process_file_future = tokio::spawn(process_file_future);

    let addr = "127.0.0.1:10000";

    // Дожидаемся успешного создания серверного сокета
    let mut listener: TcpListener = TcpListener::bind(addr)
        .await
        .unwrap();
    
    // TODO: !!! убрать, либо чистить с проверкой результата
    let mut active_futures = vec![];

    // Создаем обработчик
    let processing_sender_server = processing_sender.clone();
    let stop_read_sender_server = stop_read_sender.clone();
    let stop_write_sender_server = stop_write_sender.clone();
    let server = async move {
        // Получаем поток новых соединений
        let mut incoming: Incoming = listener.incoming();

        // Получаем кавые соединения каждый раз
        'select_loop: loop{
            tokio::select! {
                _ = stop_listen_receiver.recv() => {
                    break 'select_loop;
                }
                conn = incoming.next() => {
                    if let Some(conn) = conn {
                        match conn {
                            Ok(sock) => {
                                println!("New connection received");
            
                                // Закидываем задачу по работе с нашим сокетом в отдельную функцию
                                // tokio::spawn - неблокирующая функция, закидывающая задачу в обработку
                                let sender = processing_sender_server.clone();
                                println!("Sender cloned");
            
                                let fut = process_connection(sock, sender, 
                                    stop_read_sender_server.subscribe(), stop_write_sender_server.subscribe());
                                println!("Future created");
            
                                active_futures.push(tokio::spawn(fut));
                                // Не работает, так как фьючи могут пережить каналы и обработку данных
                                //tokio::spawn(fut);
                            }
                            Err(e) => {
                                eprintln!("accept failed = {:?}", e);
                                break 'select_loop; // TODO: ????
                            }
                        }
                    }
                }
            }
        }

        futures::future::join_all(active_futures.into_iter()).await;
    };
    
    println!("Server running on localhost:10000");
    
    let processing_sender_stop = processing_sender.clone();
    tokio::spawn(async move {
        if let Ok(_) = tokio::signal::ctrl_c().await{
            println!("Stop requested");
            stop_listen_sender.send(()).unwrap();
            stop_read_sender.send(());
            processing_sender_stop.send(ProcessCommand::Stop).unwrap();
            stop_write_sender.send(()).unwrap();
            println!("Stop request: success");
        }
    });

    // Блокируемся на выполнении сервера
    server.await;

    if let Err(e) = process_file_future.await{
        eprintln!("Process file error: {:?}", e);
    }

    println!("Application exit");

    // TODO: завершение по Ctrl + C
}