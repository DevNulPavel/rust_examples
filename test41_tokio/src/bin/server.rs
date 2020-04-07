#[allow(unused_imports)]

use std::io::Cursor;
use std::process::Output;
use std::path::{Path, PathBuf};
use std::sync::Arc;
// use futures::prelude::*;
// use tokio::prelude::*;
use futures::stream::StreamExt;
// use futures::future::FutureExt;
use tokio::process::{ Command };
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::broadcast;
//use tokio::sync::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{ TcpListener, TcpStream};
use tokio::net::tcp::{ Incoming, ReadHalf, WriteHalf };
use bytes::BytesMut;
//use rand::{Rng};
// use std::rand::{task_rng, Rng};
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
    const MAX_COUNT: usize = 8;

    // Ограничение на 8 активных задач
    let semaphore = Arc::from(tokio::sync::Semaphore::new(MAX_COUNT));

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

        // Клон семафора
        let semaphore_clone = semaphore.clone();
        tokio::spawn(async move {
            // Берем блокировку
            let acquire = semaphore_clone.acquire().await;
            // Получаем MD5 асинхронно
            if let Ok(md5_res) = get_md5(&received_path).await {
                println!("Send to channel: {}", md5_res);
                let _ = tokio::fs::remove_file(&received_path).await;
                                
                let nanos = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .subsec_nanos();
                let time: u64 = (1000 + nanos % 4000) as u64;

                tokio::time::delay_for(std::time::Duration::from_millis(time)).await;
                
                let _ = response_ch.send((received_path, md5_res)); // TODO: ???
                println!("Send to channel success");
            }
            drop(acquire);
        });
    }

    // Перехватываем блокировку уже в текущем потоке
    let waits: Vec<_> = (0..MAX_COUNT)
        .map(|_|{
            semaphore.acquire()
        })
        .collect();
    futures::future::join_all(waits).await;

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
                        println!("Process sending exit");
                        return Err("Channel closed".into());
                    }
                };
                data
            }
            _ = stop_receiver.recv() => {
                println!("Process sending exit");
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
                println!("Process incoming exit");
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

        let id: String = format!("{}", uuid::Uuid::new_v4());
        let file_path = PathBuf::new()
            .join("tmp")
            .join(id + &json.file_name + ".tmp");

        println!("File path: {:?}", file_path);            
        let save_result = save_file_from_socket(&mut reader, json.file_size, &file_path).await;
        if save_result.is_ok() {
            println!("File received: {:?}", save_result);
            let command = ProcessCommand::Process((file_path, sock_channel.clone()));
            process_sender.send(command)?;
        }else{
            eprintln!("File receive error: {:?}", save_result);
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

    println!("Process connection exit");
}

fn create_processing() -> (impl futures::Future<Output = EmptyResult>, PathSender) {
    let (processing_sender, input_receiver) = unbounded_channel::<ProcessCommand>();
    
    let process_file_future = process_files(input_receiver);

    (process_file_future, processing_sender)
}

#[tokio::main]
async fn main() {
    // Каналы остановки работы
    let (mut stop_listen_sender, mut stop_listen_receiver) = tokio::sync::mpsc::channel::<()>(1);
    let (stop_read_sender, _) = tokio::sync::broadcast::channel::<()>(1);
    let (stop_write_sender, _) = tokio::sync::broadcast::channel::<()>(1);
    
    // Канал и Join обработки данных
    let (process_file_future, processing_sender) = create_processing();
    let process_file_future = tokio::spawn(process_file_future);

    // Дожидаемся успешного создания серверного сокета
    let mut listener: TcpListener = TcpListener::bind("127.0.0.1:10000")
        .await
        .unwrap();
    
    // Создаем обработчик
    let processing_sender_server = processing_sender.clone();
    let stop_read_sender_server = stop_read_sender.clone();
    let stop_write_sender_server = stop_write_sender.clone();
    let server = async move {
        let mut active_processings: Vec<tokio::sync::oneshot::Receiver<()>> = vec![];

        // Получаем поток новых соединений
        let mut incoming: Incoming = listener.incoming();

        // Получаем кавые соединения каждый раз
        'select_loop: loop{
            let sock: TcpStream = tokio::select! {
                // Новое подключение
                conn = incoming.next() => {
                    // Новое подключение может быть пустым
                    match conn {
                        Some(conn) => {
                            // непустое подключение может быть ошибкой
                            match conn {
                                Ok(sock) => {
                                    sock
                                },
                                Err(e) => {
                                    eprintln!("accept failed = {:?}", e);
                                    break 'select_loop; // TODO: ????
                                }
                            }
                        },
                        None => {
                            continue 'select_loop;
                        }
                    }
                },
                // Либо сигнал об остановке приема новых подключений
                _ = stop_listen_receiver.recv() => {
                    break 'select_loop;
                }
            };

            // Очищаем список активных задач от уже завершившихся
            let mut new_active_futures = vec![];
            for mut receiver in active_processings.into_iter() {
                match receiver.try_recv() {
                    Ok(_) => {
                    },
                    Err(_) => {
                        new_active_futures.push(receiver);
                    }
                }
            }
            active_processings = new_active_futures;

            println!("New connection received");

            // Создаем свою копию канала отправителя задач
            let sender = processing_sender_server.clone();
            println!("Sender cloned");

            // Создаем задачу обработки соединения
            let fut = process_connection(sock, 
                                                  sender, 
                                              stop_read_sender_server.subscribe(), 
                                             stop_write_sender_server.subscribe());
            let (sender, receiver) = tokio::sync::oneshot::channel::<()>();
            tokio::spawn(async move {
                fut.await;
                sender.send(()).unwrap();
            });
            active_processings.push(receiver); // TODO: Удаление из списка
            println!("Future created");
        }

        // Ждем завершения всех обработок соединений
        for receiver in active_processings.into_iter() {
            receiver.await.unwrap();
        }
    };
    
    println!("Server running on localhost:10000");
    
    let processing_sender_stop = processing_sender.clone();
    tokio::spawn(async move {
        if let Ok(_) = tokio::signal::ctrl_c().await{
            println!("Stop requested");
            // Отключаем прием новых соединений
            stop_listen_sender.send(()).await.unwrap();

            // Отключаем чтение новых данных из сокетов
            let _ = stop_read_sender.send(());

            // Завершаем обработку и ждем завершения
            processing_sender_stop.send(ProcessCommand::Stop).unwrap();
            if let Err(e) = process_file_future.await{
                eprintln!("Process file error: {:?}", e);
            }

            // Завершаем все записи клиентам
            stop_write_sender.send(()).unwrap();
            println!("Stop request: success");
        }
    });

    // Блокируемся на выполнении сервера
    server.await;

    println!("Application exit");

    // TODO: завершение по Ctrl + C
}