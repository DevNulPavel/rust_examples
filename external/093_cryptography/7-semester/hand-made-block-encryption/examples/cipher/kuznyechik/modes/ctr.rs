mod common {
    include!("../../../common/utils.rs");
}

use common::format_u8_hex;
use hand_made_block_encryption::{cipher::kuznyechik::cipher::Kuznyechik, mode::CTR, traits::Encryptor};

fn main() {
    let key: [u8; 32] = [
        0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
        0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    ];
    let iv: [u8; 8] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    ];
    let data = &[0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10];

    let cipher = Box::new(Kuznyechik::new(&key).unwrap());
    let mode = Box::new(CTR::new(iv.to_vec()));

    let encryptor = Encryptor::new_stream(cipher, mode).unwrap();

    let encrypted: Vec<u8> = encryptor.encrypt(data).unwrap();
    let decrypted: Vec<u8> = encryptor.decrypt(&encrypted).unwrap();

    let mut string = String::new();

    string.push_str(&format!("Сообщение ({}):", data.len()));
    string.push_str(&format_u8_hex(data));
    string.push_str(&format!("Вектор инициализации ({}):", iv.len()));
    string.push_str(&format_u8_hex(&iv));
    string.push_str(&format!("Ключ ({}):", key.len()));
    string.push_str(&format_u8_hex(&key));
    string.push_str(&format!("Зашифрованное сообщение ({}):", encrypted.len()));
    string.push_str(&format_u8_hex(&encrypted));
    string.push_str(&format!("Расшифрованное сообщение ({}):", decrypted.len()));
    string.push_str(&format_u8_hex(&decrypted));

    print!("{}", string);
}
