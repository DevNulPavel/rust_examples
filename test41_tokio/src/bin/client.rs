use std::path::PathBuf;
use std::sync::Arc;
use tokio::prelude::*;
use tokio::net::TcpStream;
use tokio::net::tcp::{ ReadHalf, WriteHalf };
use tokio::sync::{Semaphore, SemaphorePermit};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender, UnboundedReceiver};
use test41_tokio::write_file_to_socket;

async fn process_sending<'a, 'b>(mut writer: WriteHalf<'a>, semaphoe: &'b Semaphore, sender: UnboundedSender<SemaphorePermit<'b>>){
    loop{
        let file_path = PathBuf::new()
            .join("test.txt");
    
        write_file_to_socket(&mut writer, &file_path)
            .await
            .unwrap();

        println!("Send success");

        let permit = semaphoe.acquire().await;
        sender.send(permit).unwrap();

        //tokio::time::delay_for(std::time::Duration::from_millis(250)).await;
    }
}

async fn process_receiving<'a, 'b>(mut reader: ReadHalf<'a>, _: &'b Semaphore, mut receiver: UnboundedReceiver<SemaphorePermit<'b>>){
    let mut buffer: [u8; 256] = [0; 256];
    loop{
        if let Ok(read_size) = reader.read(&mut buffer).await{
            let data: Vec<u8> = buffer[0..read_size].into();
            println!("Read from stream; success={:?}", String::from_utf8(data).unwrap());
        }else{
            break;
        }

        let permit = receiver.recv().await;
        drop(permit);
    }
}

#[tokio::main]
async fn main() {
    let semaphore = Semaphore::new(16);
    let (sender, receiver) = unbounded_channel();

    let mut stream: TcpStream = TcpStream::connect("127.0.0.1:10000")
        .await
        .unwrap();
    println!("created stream");

    let (reader, writer) = stream.split();
    let send_handle = process_sending(writer, &semaphore, sender);
    let receive_handle = process_receiving(reader, &semaphore, receiver);
    
    tokio::join!(send_handle, receive_handle);
}