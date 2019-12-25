extern crate thread_pool;

use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs::File;
use std::thread;
use std::time::Duration;

use thread_pool::ThreadPool;

// https://doc.rust-lang.ru/book/ch20-06-graceful-shutdown-and-cleanup.html


fn handle_connection(mut stream: TcpStream) {
    // Инициализируем буффер нулями, буффер на 512 значений
    let mut buffer = [0; 512];

    // Начинаем читать из потока
    if let Err(err) = stream.read(&mut buffer) {
        println!("Failed to read from socket: {}", err);
        return;
    }
    
    // Здесь описывается GET запрос + path + версия HTTP
    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    // Получаем значения, которые будем выводить, заголовок + файлик контента
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "res/hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK\r\n\r\n", "res/hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "res/404.html")
    };
    
    // Открываем файлик и читаем его содержимое
    let mut file = match File::open(filename) {
        Ok(file) => file,
        Err(_) => {
            println!("Failed to read file with path: {}", filename);
            return;
        }
    };

    // Читаем в файлик
    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err(){
        println!("Failed to read file content: {}", filename);
        return;
    }
    
    // Форматируем содержимое
    let response = format!("{}{}", status_line, contents);
    
    // Пишем данные в ответ от сервера
    if stream.write(response.as_bytes()).is_err(){
        println!("Failed to write to socket");
        return;
    }

    // Сбрасывем данные
    if stream.flush().is_err(){
        println!("Failed to flush socket");
        return;
    }
}

fn main() {
    // Создаем новый лиснейр
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    
    // Создаем пул потоков для обработки
    let pool = ThreadPool::new(4, 30);
    
    // Счетчик для отключения
    let mut counter = 0;
    
    // Начинаем обрабатывать входящие подключения
    for stream in listener.incoming() {
        // Если счетчик подключений развен 2м - выходим
        if counter == 2 {
            println!("Shutting down.");
            break;
        }
        
        counter += 1;
        
        let stream = match stream{
            Ok(val) => val,
            Err(_) => {
                println!("Stream get failed");
                break;
            }
        };
        
        // Закидываем в пул потоков данные
        pool.execute(|| {
            handle_connection(stream);
        }).unwrap();
    }
}