use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, stdin, BufReader, AsyncBufReadExt};
use serde::{Serialize, Deserialize};
use std::io::{self, Write};

#[derive(Serialize, Deserialize, Debug)]
enum Message {
    Register,
    RequestKey {user_id: usize, key: u32},
    ResponseKey {server_data: u32, users_id: Vec<usize>},
    RequestData{from: usize},
    ResponseData { from: usize, to: usize, data: u32},
    Error{message: String},
    // id пользователя который создал дату, и сама дата
}
#[derive(Serialize, Deserialize, Debug)]
struct UserData {
    pub user_id: usize,
    pub p: u32,
    pub k: u32,
}

#[tokio::main]
async fn main() {
    let mut socket = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    let (mut reader, mut writer) = socket.split();

    let register_msg = Message::Register;
    let serialized = serde_json::to_vec(&register_msg).unwrap();
    writer.write_all(&serialized).await.unwrap();

    let mut input_buffer = vec![0;1024];
    let size = reader.read(&mut input_buffer).await.unwrap();
    let user_data: UserData = serde_json::from_slice(&input_buffer[0..size]).unwrap();
    println!("Часть секрета: {}", user_data.k);

    let mut buffer = vec![0; 1024];
    let mut input = String::new();
    let mut stdin = BufReader::new(stdin());
    let mut lines = stdin.lines();

    loop {
        println!("Введите 1 для восстановления секрета!");
        io::stdout().flush().unwrap();
        tokio::select! {
            line = lines.next_line() => {
                if let Ok(Some(mut line)) = line {
                    match line.trim() {
                    "1" => {
                        let request_message = serde_json::to_vec(&Message::RequestKey {user_id: user_data.user_id, key: user_data.k}).unwrap();
                        writer.write_all(&request_message).await.unwrap();
                    },
                    _ => {},
                }
                }
                //println!("{input}");
            }
            n = reader.read(&mut buffer)=> {
                 let n = n.unwrap();
                if n == 0 {
                    break;
                }
                 let msg: Message = serde_json::from_slice(&buffer[..n]).unwrap();

                match msg {
                    Message::Error {message} => {
                        println!("Server error: {message}");
                    }
                    Message::ResponseKey {server_data, users_id} => {
                        println!("server key: {server_data}");
                    }
                    Message::RequestData {from} => {
                        writer.write_all(&serde_json::to_vec(&Message::ResponseData {from: user_data.user_id, data: user_data.k, to: from}).unwrap()).await.unwrap();
                    }
                    _ => ()
                }
            }
        }
    }
}

