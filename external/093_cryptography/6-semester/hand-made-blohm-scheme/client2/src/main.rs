use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use nalgebra::DMatrix;
use server::{Data, MatrixWrapper};

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:9090").unwrap();
    let mut buffer = [0u8;1024];

    let size = stream.read(&mut buffer).unwrap();
    let data: Data = serde_json::from_str(&String::from_utf8_lossy(&buffer[..size])).unwrap();

    let self_open_key: DMatrix<u32> = data.get_open_key().into();
    let self_close_key: DMatrix<u32> = data.get_close_key().into();
    let p = data.get_p();

    println!("Ключи получены!");
    println!("Введите 1 для получения сессиного ключа с другим пользователем!");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    match input.trim().parse::<usize>() {
        Ok(1) => {
            println!("Отправляем открытый ключ второму пользователю");
            let listener = TcpListener::bind("127.0.0.1:9191").unwrap();
            while let Ok((mut stream_client_2, _)) = listener.accept() {
                let mut buffer = [0u8;1024];
                let size = stream_client_2.read(&mut buffer).unwrap();
                stream_client_2.write_all(serde_json::to_string(&MatrixWrapper::from(&self_open_key)).unwrap().as_bytes()).unwrap();
                let open_key_client_2: MatrixWrapper = serde_json::from_str(&String::from_utf8_lossy(&buffer[..size])).unwrap();
                let session_key = self_close_key.transpose() * <MatrixWrapper as Into<DMatrix<u32>>>::into(open_key_client_2);
                let session_key = session_key.map(|el| el % p);
                println!("Key: {}", session_key.data.as_vec()[0]);
            }
        }
        Ok(_) => (),
        Err(e) => println!("error_parse: {e}")
    }
}

