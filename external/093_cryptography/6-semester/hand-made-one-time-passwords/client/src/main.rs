use std::io::Write;
use rand::random;
use std::net::TcpStream;
use client::Data;

fn main() {
    let secret_key = "secret_key";
    let count_hashes: usize = 9000;
    let mut current_identification = 1;
    let mut stream = TcpStream::connect("127.0.0.1:9090").unwrap();

    let mut hash = hand_made_sha::sha256(secret_key.as_bytes());
    let mut hashes = vec![hash];

    for _ in 1..count_hashes {
        hash = hand_made_sha::sha256(&hash);
        hashes.push(hash)
    }
    // передаём последний хеш серверу
    stream.write_all(&hashes[count_hashes-1]).unwrap();

    loop {
        let mut input = String::new();
        println!("Введите 1 если хотите аутентифицироваться. _ - для выхода");
        std::io::stdin().read_line(&mut input).unwrap();
        match input.trim().parse::<usize>() {
            Ok(1) => {
                let data = Data::new("A".to_string(), current_identification, hashes[count_hashes-current_identification-1]);
                let data_to_string = serde_json::to_string(&data).unwrap();
                stream.write_all(data_to_string.as_bytes()).unwrap();
                current_identification += 1;

                if current_identification == count_hashes {
                    break;
                }
            }
            Ok(_) => break,
            Err(e) => {
                println!("Ошибка ввода: {e}");
                break
            }
        }
    }

}
