use std::{
    fs::{
        File
    }, 
    io::{
        Read
    }, 
    path::{
        Path
    }
};

pub enum ImageFileType {
    WebP,
    Png,
    Jpg
}

/// Получаем тип картинки или None если не картинка
pub fn get_image_file_type(path: &Path) -> Option<ImageFileType> {
    let mut file = File::open(path)
        .expect("File open failed");

    // Буффер, который будем вычитывать
    let mut first_bytes: [u8; 16] = [0; 16];

    // Проверяем заранее, можем ли мы вычитать нужные нам данные
    if file.metadata().expect("Metadata read failed").len() < first_bytes.len() as u64 {
        return None;
    }

    file.read_exact(&mut first_bytes).expect("File bytes read failed");

    drop(file);

    // https://ru.wikipedia.org/wiki/%D0%A1%D0%BF%D0%B8%D1%81%D0%BE%D0%BA_%D1%81%D0%B8%D0%B3%D0%BD%D0%B0%D1%82%D1%83%D1%80_%D1%84%D0%B0%D0%B9%D0%BB%D0%BE%D0%B2
    if first_bytes.starts_with(&[0x52, 0x49, 0x46, 0x46]) {
        Some(ImageFileType::WebP)
    } else if first_bytes.starts_with(&[0xFF, 0xD8]) { // 0xFF, 0xE1
        Some(ImageFileType::Jpg)
    } else if first_bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some(ImageFileType::Png)
    } else {
        None
    }
}