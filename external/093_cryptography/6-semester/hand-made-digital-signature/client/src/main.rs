mod utils;
use crate::utils::{get_data_from_menu, send_file, DataInput, Menu};
use pkcs7::PKCS7;
use std::io::Read;
use std::net::TcpStream;

fn main() {
    let mut file_name = "pkcs7.json";
    let mut message = String::new();
    println!("Введите текст, который необходимо зашифровать:");
    std::io::stdin().read_line(&mut message).unwrap();

    loop {
        match call_menu() {
            Ok(data_input) => {
                // Получение типа Хеш функции и Системы шифрования с ввода пользователя
                let (hash_function, encryption_function) = data_input.get_data();

                dbg!(&hash_function.get_id());

                // Формирование всех начальных необходимых данных для документа
                let pkcs7 = PKCS7::new_pkcs7(
                    encryption_function.as_ref(),
                    hash_function.as_ref(),
                    &message,
                    "digit4lsh4d0w".to_string(),
                );
                // Создаем json файл документа
                pkcs7.save_to_json(file_name);

                dbg!(&pkcs7);

                println!("🟡 Попытка подключения к серверу ...");
                let mut stream = TcpStream::connect("127.0.0.1:8090").expect("🔴 Таймаут");
                println!("🟢 Соединение установлено!");
                let mut input = String::new();

                send_file(&stream, file_name);
                stream.read_to_string(&mut input).unwrap();
                let server_pkcs7: PKCS7 = serde_json::from_str(&input).unwrap();

                println!("Ответ сервера получен, штамп времени поставлен!");
                file_name = "pkcs7_server.json";
                server_pkcs7.save_to_json(file_name);
            }
            Err(error) => println!("Error: {}", error),
        }
    }
}

/// Функция вызова меню, возвращает данные введённые пользователем, иначе - ошибку ввода
fn call_menu() -> Result<DataInput, Box<dyn std::error::Error>> {
    let mut menu = Menu::Main;
    let mut data_input = DataInput::default();

    loop {
        match get_data_from_menu(menu, &mut data_input) {
            Ok(Some(menu_type)) => {
                if menu_type == Menu::MenuEnd {
                    break;
                } else {
                    menu = menu_type
                }
            }
            Err(error) => return Err(error),
            _ => (),
        }
    }

    Ok(data_input)
}
