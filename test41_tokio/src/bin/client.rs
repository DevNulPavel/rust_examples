use std::{
    path::PathBuf,
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
        oneshot::{
            channel as oneshot_channel,
            Receiver as OneshotReceiver,
            //Sender as OneshotSender
        },
        mpsc::{
            unbounded_channel, 
            UnboundedSender, 
            UnboundedReceiver
        }
    }
};
use test41_tokio::socket_helpers::write_file_to_socket;

#[derive(Debug)]
enum ProcessingMessage<'b>{
    Permit(SemaphorePermit<'b>),
    Complete(SemaphorePermit<'b>),
}

async fn process_sending<'a, 'b>(mut writer: WriteHalf<'a>, 
                                 semaphoe: &'b Semaphore, 
                                 mut exit_channel: OneshotReceiver<()>,
                                 sender: UnboundedSender<ProcessingMessage<'b>>){
    loop{
        // Путь к отправляемому файлику
        let file_path = PathBuf::new()
            .join("test.txt");
    
        // Пишем файлик в сокет
        write_file_to_socket(&mut writer, &file_path)
            .await
            .unwrap();

        println!("Send success");

        let permit = semaphoe.acquire().await;

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
                .send(ProcessingMessage::Complete(permit))
                .unwrap();
            break;
        }
    }
}

async fn process_receiving<'a, 'b>(mut reader: ReadHalf<'a>, 
                                   mut receiver: UnboundedReceiver<ProcessingMessage<'b>>){
    let mut buffer: [u8; 256] = [0; 256];
    loop{
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

        let message = receiver.recv().await.unwrap();
        match message {
            ProcessingMessage::Permit(perm)=>{
                drop(perm)
            },
            ProcessingMessage::Complete(perm) => {
                drop(perm);
                println!("Stop received");
                break;
            }
        }
    }
}

// Клиент будет однопоточным, чтобы не отжирать бестолку ресурсы
#[tokio::main(core_threads = 1)]
async fn main() {
    // Подключаемся к серверу
    let mut stream: TcpStream = TcpStream::connect("127.0.0.1:10000")
        .await
        .unwrap();
    println!("Сreated stream");

    let (exit_sender, exit_receiver) = oneshot_channel();

    let (reader, writer) = stream.split();

    let semaphore = Semaphore::new(16);
    let (sender, receiver) = unbounded_channel();

    let send_handle = process_sending(writer, &semaphore, exit_receiver, sender);
    let receive_handle = process_receiving(reader, receiver);
    
    // Обработка сигнала прерывания работы по Ctrl+C
    tokio::spawn(async move {
        if let Ok(_) = tokio::signal::ctrl_c().await {
            println!("Stop requested");
            exit_sender.send(()).unwrap();
        }
    });

    // Блокируемся на ожидании завершения
    tokio::join!(send_handle, receive_handle);

    println!("Completed");
}