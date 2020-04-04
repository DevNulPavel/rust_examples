use std::path::{Path, PathBuf};
use tokio::prelude::*;
use tokio::net::TcpStream;
use test41_tokio::{write_file_to_socket, EmptyResult};

#[tokio::main]
async fn main() {
    let mut stream: TcpStream = TcpStream::connect("127.0.0.1:10000")
        .await
        .unwrap();
    println!("created stream");
    
    let mut buffer: [u8; 256] = [0; 256];

    loop {
        let (mut reader, mut writer) = stream.split();

        let file_path = PathBuf::new()
            .join("Cargo.toml");
        write_file_to_socket(&mut writer, &file_path)
            .await
            .unwrap();
        
        println!("Write success");

        if let Ok(read_size) = reader.read(&mut buffer).await{
            let data: Vec<u8> = buffer[0..read_size].into();
            println!("Read from stream; success={:?}", String::from_utf8(data).unwrap());
        }else{
            break;
        }

        println!("Read success");

        tokio::time::delay_for(std::time::Duration::from_millis(1000)).await;
    }
}