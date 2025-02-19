mod common {
    include!("../../../common/utils.rs");
}

use common::format_u8_hex;
use hand_made_block_encryption::{cipher::magma::{Magma, ID_TC26_GOST_28147_PARAM_Z}, mode::CBC, padding::PKCS7, traits::Encryptor};

fn main() {
    let key: [u8; 32] = [
        0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88,
        0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00,
        0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7,
        0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
    ];
    let iv: [u8; 8] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    ];
    let data = &[0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10];

    let cipher = Box::new(Magma::new(&key, ID_TC26_GOST_28147_PARAM_Z).unwrap());
    let padding = Box::new(PKCS7);
    let mode = Box::new(CBC::new(iv.to_vec()));

    let encryptor = Encryptor::new_block(cipher, mode, padding).unwrap();

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
