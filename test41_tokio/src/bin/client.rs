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
    sync::{
        Semaphore, 
        SemaphorePermit,
        Mutex,
        //oneshot::{
            //channel as oneshot_channel,
            //Receiver as OneshotReceiver,
            //Sender as OneshotSender
        //},
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
            //Sender
        }
    }
};
use test41_tokio::socket_helpers::write_file_to_socket;

const MAX_PROCESSING_FILES_PER_ADDRESS: usize = 16;

#[derive(Debug)]
enum ProcessingMessage<'b>{
    Permit(SemaphorePermit<'b>),
    //Stop(SemaphorePermit<'b>),
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
                         tasks: Arc<Mutex<Receiver<PathBuf>>>, 
                         exit_receiver: BroadcastReceiver<()>) {
    // Подключаемся к серверу
    let mut stream: TcpStream = TcpStream::connect(addr)
        .await
        .unwrap();
    println!("Сreated stream");

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
        "127.0.0.1:10000"
    ];

    // Канал для выхода из всех обработчиков
    let (exit_sender, _) = broadcast_channel(ADDRESSES.len());

    let (mut tasks_sender, tasks_receiver_raw) = channel(MAX_PROCESSING_FILES_PER_ADDRESS * ADDRESSES.len());
    let tasks_receiver = Arc::new(Mutex::new(tasks_receiver_raw));

    // Фьючи соединений с адресами, запуск в работу будет ниже при вызове await
    let process_futures_join = tokio::spawn(futures::future::join_all(ADDRESSES
        .iter()
        .zip(std::iter::repeat_with(|| {
            (tasks_receiver.clone(), exit_sender.subscribe())
        }))
        .map(|(addr, (task_receiver, exit_receiver))|{
            process_address(addr, task_receiver, exit_receiver)
        })));

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