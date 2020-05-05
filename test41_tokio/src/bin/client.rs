use std::{
    path::{
        //Path,
        PathBuf
    },
    sync::{
        Arc,
        //Mutex
    }
    //net::Shutdown
};
use tokio::{
    prelude::*,
    net::{
        TcpStream,
        tcp::{
            ReadHalf, 
            WriteHalf
        }
    },
    time::{
        timeout,
        Duration
    },
    sync::{
        Semaphore, 
        SemaphorePermit,
        Mutex,
        // Notify,
        // Barrier,
        // oneshot::{
        //     channel as oneshot_channel,
        //     Receiver as OneshotReceiver,
        //     Sender as OneshotSender
        // },
        broadcast::{
            channel as broadcast_channel,
            Receiver as BroadcastReceiver
        },
        mpsc::{
            unbounded_channel, 
            UnboundedSender, 
            UnboundedReceiver,
            channel,
            Receiver,
            Sender
        }
    }
};
use test41_tokio::socket_helpers::write_file_to_socket;

const MAX_PROCESSING_FILES_PER_ADDRESS: usize = 16;
const CONNECTION_TIMEOUT_IN_SEC: u64 = 5;

#[derive(Debug)]
enum ProcessingMessage<'b>{
    Permit(SemaphorePermit<'b>),
    Finished(),
}

async fn process_sending<'a, 'b>(mut writer: WriteHalf<'a>, 
                                 semaphoe: &'b Semaphore, 
                                 tasks: Arc<Mutex<Receiver<PathBuf>>>,
                                 mut exit_channel: BroadcastReceiver<()>,
                                 sender: UnboundedSender<ProcessingMessage<'b>>){
    loop{
        let file_path = {
            let mut receiver = tasks.lock().await;
            if let Some(path) = receiver.recv().await {
                path
            }else{
                println!("Empty data from tasks - exit");
                writer
                    .shutdown()
                    .await
                    .unwrap();
                sender
                    .send(ProcessingMessage::Finished())
                    .unwrap();
                break;
            }
        };
    
        let permit = semaphoe.acquire().await;

        // Пишем файлик в сокет
        write_file_to_socket(&mut writer, &file_path)
            .await
            .unwrap();

        println!("Send success");

        if exit_channel.try_recv().is_err(){
            // Чтобы ограничить максимальное количество необработанных файликов,
            // создаем новый объект семафора, который надо дропнется уже при принятии файлика
            // Таким образом мы ограничиваем максимальное количество файлов на обработке на сервере
            sender
                .send(ProcessingMessage::Permit(permit))
                .unwrap();
        }else{
            writer.shutdown().await.unwrap();
            sender
                .send(ProcessingMessage::Finished())
                .unwrap();
            break;
        }
    }
    println!("Sending exit");
}

async fn process_receiving<'a, 'b>(mut reader: ReadHalf<'a>, 
                                   mut receiver: UnboundedReceiver<ProcessingMessage<'b>>){
    let mut buffer: [u8; 256] = [0; 256];
    loop{
        let _permit = match receiver.recv().await.unwrap() {
            ProcessingMessage::Permit(perm)=>{
                perm
            },
            ProcessingMessage::Finished() =>{
                println!("Finished received");
                break;
            }
        };

        let data_size: u16 = match reader.read_u16().await{
            Ok(size) if (size as usize <= buffer.len()) => size,
            Ok(_invalid_size) => { break; },
            Err(_e) => { break; }
        };
        
        // Читаем данные из сокета
        if let Ok(read_size) = reader.read_exact(&mut buffer[0..data_size as usize]).await {
            // Превращаем наши вычитанные данные в вектор
            //let data: Vec<u8> = .into();
            println!("Read from stream; success={:?}", String::from_utf8_lossy(&buffer[0..read_size]));
        }else{
            break;
        }
    }
    println!("Receiving exit");
}

async fn process_address(addr: &str, 
                         mut connected: Sender<bool>,
                         tasks: Arc<Mutex<Receiver<PathBuf>>>, 
                         exit_receiver: BroadcastReceiver<()>) {
    // Подключаемся к серверу
    let mut stream: TcpStream = match timeout(Duration::from_secs(CONNECTION_TIMEOUT_IN_SEC), TcpStream::connect(addr)).await{
        Ok(stream_res) => {
            match stream_res {
                Ok(stream) => {
                    stream
                },
                Err(e) => {
                    println!("Connect to address error: {} ({})", addr, e);
                    connected.send(false).await.unwrap();
                    return;
                }
            }
        },
        Err(e) => {
            println!("Connect timeout to address: {} ({})", addr, e);
            connected.send(false).await.unwrap();
            return;
        }
    };

    println!("Stream created");

    connected.send(true).await.unwrap();

    let (reader, writer) = stream.split();

    let semaphore = Semaphore::new(MAX_PROCESSING_FILES_PER_ADDRESS);
    let (sender, receiver) = unbounded_channel();

    let send_handle = process_sending(writer, &semaphore, tasks, exit_receiver, sender);
    let receive_handle = process_receiving(reader, receiver);

    tokio::join!(send_handle, receive_handle);
}

// Клиент будет однопоточным, чтобы не отжирать бестолку ресурсы
#[tokio::main(core_threads = 1)]
async fn main() {
    const ADDRESSES: [&str; 2] = [
        "127.0.0.1:10000",
        "127.0.0.1:10001",
    ];

    // Канал для выхода из всех обработчиков
    let (exit_sender, _) = broadcast_channel(ADDRESSES.len());

    // Канал отправки задач
    let (mut tasks_sender, tasks_receiver_raw) = channel(MAX_PROCESSING_FILES_PER_ADDRESS * ADDRESSES.len());
    let tasks_receiver = Arc::new(Mutex::new(tasks_receiver_raw));

    // Переменая установки соединения
    let (connection_established_sender, mut connection_established_receiver)  = channel(ADDRESSES.len());

    // Фьючи соединений с адресами, запуск в работу будет ниже при вызове await
    let process_futures_join = {
        let connections_iter = ADDRESSES
            .iter()
            .zip(std::iter::repeat_with(|| {
                (connection_established_sender.clone(), tasks_receiver.clone(), exit_sender.subscribe())
            }))
            .map(move |(addr, (notify, task_receiver, exit_receiver))|{
                process_address(addr, notify, task_receiver, exit_receiver)
            });

        let futures_join = futures::future::join_all(connections_iter);
        let join = tokio::spawn(futures_join);
        
        join
    };

    // Результат установки хоть какого-то соединения, продолжаем работу только если установили соединение
    {
        let connected = {
            let mut conn = false;
            for _ in 0..ADDRESSES.len(){
                let connect_success = connection_established_receiver.recv().await.unwrap();
                if connect_success {
                    conn = true;
                    break;
                }
            }
            conn
        };
        if connected == false {
            println!("Connection failed");
            return;
        }
        drop(connection_established_sender);
        drop(connection_established_receiver);
    }

    // Производитель задач
    let tasks_join = {
        let mut exit_receiver = exit_sender.subscribe();
        tokio::spawn(async move {
            for _ in 0..100 {
                //println!("Send task");
                let path = PathBuf::new().join("test.txt");

                // else ветка почему-то не захотела работать
                tokio::select! {
                    _ = exit_receiver.recv() => {
                        break;
                    }
                    _ = tasks_sender.send(path) => {
                    }
                }
            }
            drop(tasks_sender);
            println!("Task producer exit");
        })
    };

    // Обработка сигнала прерывания работы по Ctrl+C
    tokio::spawn(async move {
        if let Ok(_) = tokio::signal::ctrl_c().await {
            println!("Stop requested");
            exit_sender.send(()).unwrap();
        }
    });

    process_futures_join.await.unwrap();
    tasks_join.await.unwrap();

    println!("Completed");
}


// TODO:
// - может быть избавиться от Mutex?
// - таймауты на работу с сокетами
// - если отвалился один из серверов по таймауту или по другой причне, 
//      надо вернуть полученные им задачи снова в очередь, чтобы отработали другие сервера
