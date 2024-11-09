use std::collections::HashMap;
use std::io::Read;
use std::net::TcpListener;
use hand_made_rsa::keys::{PrivateKey, PublicKey};
use hand_made_rsa::{RSA, TypeKey};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9090").unwrap();
    let mut buffer = [0u8;8000];

    while let Ok((mut stream, _)) = listener.accept() {

        let size = stream.read(&mut buffer).unwrap();
        let data = String::from_utf8_lossy(buffer[..size].as_ref());
        let mut data = data.split("|");
        let public_key: HashMap<String, String> = serde_json::from_str(data.next().unwrap()).unwrap();
        let private_key: HashMap<String, String> = serde_json::from_str(data.next().unwrap()).unwrap();

        let rsa = RSA::create(PrivateKey::create_key_from_hashmap(private_key), PublicKey::create_key_from_hashmap(public_key));

        let size = stream.read(&mut buffer).unwrap();
        let encrypted_message = String::from_utf8_lossy(&buffer[..size]).to_string();
        let decrypted_message = rsa.decrypt_msg(TypeKey::PrivateKey, &encrypted_message).split("|").map(|el| el.to_string()).collect::<Vec<String>>();

        let message = vec![decrypted_message[1].clone(), decrypted_message[0].clone(), "B".to_string()].join("|");

        let message_from_sign = rsa.decrypt_msg(TypeKey::PublicKey, &decrypted_message[2]);

        if message == message_from_sign {
            println!("Подпись проверена, аутентификация выполнена успешна!");
        }

        let split_message_from_sign = message_from_sign.split("|").map(|el| el.to_string()).collect::<Vec<String>>();

        println!("Сессионый ключ :{}, случайное число: {}, значение подписи: {}", decrypted_message[0], decrypted_message[1], decrypted_message[2]);
        println!("Данные цифровой подписи. Случайное число: {}, сессионый ключ: {}, идентификатор пользователя: {}", split_message_from_sign[0], split_message_from_sign[1], split_message_from_sign[2]);
    }
}
