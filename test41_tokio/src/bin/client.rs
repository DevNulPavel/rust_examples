use tokio::prelude::*;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let mut stream: TcpStream = TcpStream::connect("127.0.0.1:6142").await.unwrap();
    println!("created stream");
    
    let mut buffer: [u8; 256] = [0; 256];

    loop {
        let result = stream.write(b"hello world\n").await;
        println!("wrote to stream; success={:?}", result.is_ok());
        if !result.is_ok() {
            break;
        }

        let read_result = stream.read(&mut buffer).await;
        println!("Read from stream; success={:?}", read_result.is_ok());
        if !read_result.is_ok() {
            break;
        }
    }
}