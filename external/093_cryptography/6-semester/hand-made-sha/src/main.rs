use hand_made_sha::{sha256, sha512};

fn main() {
    let message = "привет мир\n";
    let hash256 = sha256(message.as_bytes());
    let hash512 = sha512(message.as_bytes());
    let hash256str = hash256.iter().map(|byte| format!("{:08x}", byte)).collect::<Vec<String>>().join("");
    let hash512str = hash512.iter().map(|byte| format!("{:016x}", byte)).collect::<Vec<String>>().join("");

    println!("Сообщение:  {}", message.trim());
    println!("Хэш SHA256: {}", hash256str);
    println!("Хэш SHA512: {}", hash512str);
}
