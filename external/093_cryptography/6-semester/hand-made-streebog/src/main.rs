use std::{fs, path::Path};

use hand_made_streebog::{streebog_256, streebog_512};

fn main() {
    let message = "hello world".as_bytes();

    let hash512 = streebog_512(message);
    let hash512str = hash512.iter().map(|byte| format!("{:02x}", byte)).collect::<Vec<String>>().join("");
    let hash256 = streebog_256(message);
    let hash256str = hash256.iter().map(|byte| format!("{:02x}", byte)).collect::<Vec<String>>().join("");

    println!("Streebog 512: {}", hash512str);
    println!("Streebog 256: {}", hash256str);

    let data = fs::read(Path::new("./assets/video.mp4")).expect("Ошибка чтений файла");
    let _hash = streebog_512(data.as_slice());
    let _hash = streebog_256(data.as_slice());
}
