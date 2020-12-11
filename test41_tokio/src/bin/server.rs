#[allow(unused_imports)]

use std::{
    io::Cursor,
    path::PathBuf,
    sync::{
        atomic::{
            AtomicI64,
            Ordering,
        },
        Arc
    }
};
// use std::process::Output;
// use std::sync::Arc;
// use futures::prelude::*;
// use tokio::prelude::*;
use futures::{
    future::{
        TryFuture,
        TryFutureExt,
        FutureExt
    },
    stream::StreamExt
};
use tokio::{
    sync::{
        mpsc::{
            unbounded_channel
        },
        broadcast,
        Notify,
    }
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{ TcpListener, TcpStream};
use tokio::net::tcp::{ Incoming, ReadHalf, WriteHalf };
use bytes::BytesMut;
//use rand::{Rng};
// use std::rand::{task_rng, Rng};
// use test41_tokio::*;
// use test41_tokio::errors::*;
use test41_tokio::types::*;
use test41_tokio::file_processing::Processing;
use test41_tokio::socket_helpers::*;

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
            },
            _ = stop_receiver.recv() => {
                println!("Process sending exit");
                return Ok(());
            }
        };

        println!("Data for send: {:?}", received);

        let data: String = received.1;
        let data_bytes = data.as_bytes();
        writer.write_u16(data_bytes.len() as u16).await?;
        writer.write_all(data_bytes).await?;
    }
}

async fn process_incoming_data<'a>(mut reader: ReadHalf<'a>,
                                   process_sender: PathSender,
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

async fn process_connection(mut sock: TcpStream, 
                            process_sender: PathSender, 
                            stop_read_receiver: broadcast::Receiver<()>, 
                            stop_write_receiver: broadcast::Receiver<()>){
    println!("Process connection: {:?}", sock);

    //let sock = std::sync::Arc::from(sock);
    // Получаем отдельные каналы сокета на чтение и на запись
    let (reader, writer) = sock.split();
    let reader: ReadHalf = reader;
    let writer: WriteHalf = writer;

    let (sender, receiver) = unbounded_channel::<(PathBuf, String)>();

    let reader_join = process_incoming_data(reader, 
                                                                     process_sender, 
                                                       sender, 
                                                      stop_read_receiver);
    let writer_join = process_sending_data(writer, 
                                                                    receiver, 
                                                     stop_write_receiver);

    let (reader_res, writer_res) = tokio::join!(reader_join, writer_join);
    if let Err(reader_res) = reader_res {
        eprintln!("{:?}", reader_res);
    }
    if let Err(writer_res) = writer_res {
        eprintln!("{:?}", writer_res);
    }

    println!("Process connection exit");
}

struct ActiveProcessings{
    counter: AtomicI64,
    notify: Notify
}

impl ActiveProcessings {
    fn new() -> ActiveProcessings{
        ActiveProcessings{
            counter: AtomicI64::new(0),
            notify: Notify::new()
        }
    }
    fn acquire(&self){
        self.counter.fetch_add(1, Ordering::AcqRel); // TODO: ???
    }
    fn release(&self){
        self.counter.fetch_sub(1, Ordering::AcqRel); // TODO: ???
        self.notify.notify();
    }
    async fn wait_finish(&self){
        while self.counter.load(Ordering::Acquire) > 0 {
            self.notify.notified().await;
        }
    }
}


// Сервер будет однопоточным, чтобы не отжирать бестолку ресурсы
#[tokio::main(core_threads = 1)]
async fn main() {
    // Каналы остановки работы
    let (mut stop_listen_sender, mut stop_listen_receiver) = tokio::sync::mpsc::channel::<()>(1);
    let (stop_read_sender, _) = tokio::sync::broadcast::channel::<()>(1);
    let (stop_write_sender, _) = tokio::sync::broadcast::channel::<()>(1);
    
    // Канал и Join обработки данных
    let processing = Processing::new();

    // Дожидаемся успешного создания серверного сокета
    let mut listener: TcpListener = TcpListener::bind("127.0.0.1:10000")
        .await
        .unwrap();
    
    // Создаем обработчик
    let processing_sender_server = processing.get_sender_clone();
    let stop_read_sender_server = stop_read_sender.clone();
    let stop_write_sender_server = stop_write_sender.clone();
    let server = async move {
        // Получаем поток новых соединений
        let mut incoming: Incoming = listener.incoming();

        let active_processings = Arc::new(ActiveProcessings::new());

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

            println!("New connection received");

            // Создаем свою копию канала отправителя задач
            let sender = processing_sender_server.clone();
            println!("Sender cloned");

            // Создаем задачу обработки соединения
            let fut = process_connection(sock, 
                                         sender, 
                                         stop_read_sender_server.subscribe(), 
                                         stop_write_sender_server.subscribe());
            active_processings.acquire();
            let active_processings_clone = active_processings.clone();
            tokio::spawn(async move{
                fut.await;
                active_processings_clone.release();
            });

            println!("Future created");
        }

        // Ждем завершения всех обработок соединений
        active_processings.wait_finish().await;
    };
    
    println!("Server running on localhost:10000");
    
    // Обработка сигнала прерывания работы по Ctrl+C
    tokio::spawn(async move {
        if let Ok(_) = tokio::signal::ctrl_c().await{
            println!("Stop requested");
            // Отключаем прием новых соединений
            stop_listen_sender.send(()).await.unwrap();

            // Отключаем чтение новых данных из сокетов
            let _ = stop_read_sender.send(());

            // Завершаем обработку и ждем завершения
            processing.gracefull_finish_and_wait().await;

            // Завершаем все записи клиентам
            stop_write_sender.send(()).unwrap();
            println!("Stop request: success");
        }
    });

    // Блокируемся на выполнении сервера
    server.await;

    println!("Application exit");
}