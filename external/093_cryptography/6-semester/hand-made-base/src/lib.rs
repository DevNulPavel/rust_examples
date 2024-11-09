//! Функции кодирования и декодирования base32/base64

/// # Функция кодирования BASE64
/// Кодирование набора байт в строку.
///
/// ## Параметры
/// `message` данные в голом виде.
///
/// ## Возвращаемые значения
/// Строка данных в кодировке BASE64.
pub fn base64_encode(content: &[u8]) -> String {
    const BASE64_ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let mut content_vec = content.to_vec();

    let padding_len = 3 - content_vec.len() % 3;
    if padding_len != 3 {
        content_vec.extend(vec![0u8; padding_len]);
    }

    for chunk in content_vec.chunks(3) {
        let tmp_buff = [
            (chunk[0] >> 2),
            ((chunk[0] << 4) & 0b00110000) | ((chunk[1] >> 4) & 0b00001111),
            ((chunk[1] << 2) & 0b00111100) | ((chunk[2] >> 6) & 0b00000011),
            (chunk[2] & 0b00111111)
        ];

        for byte in tmp_buff {
            result.push(BASE64_ALPHABET[byte as usize] as char);
        }
    }

    if padding_len != 3 {
        result.replace_range((result.len() - padding_len)..result.len(), &"=".repeat(padding_len));
    }

    result
}

/// # Функция кодирования BASE32
/// Кодирование набора байт в строку.
///
/// ## Параметры
/// `message` данные в голом виде.
///
/// ## Возвращаемые значения
/// Строка данных в кодировке BASE32.
pub fn base32_encode(content: &[u8]) -> String {
    const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut result = String::new();
    let mut content_vec = content.to_vec();
    
    let padding_len = 5 - content_vec.len() % 5;
    if padding_len != 5 {
        content_vec.extend(vec![0u8; padding_len]);
    }

    for chunk in content_vec.chunks(5) {
        let buffer = [
            (chunk[0] >> 3),
            ((chunk[0] << 2) & 0b00011100) | ((chunk[1] >> 6) & 0b00000011),
            ((chunk[1] >> 1) & 0b00011111),
            ((chunk[1] << 4) & 0b00010000) | ((chunk[2] >> 4) & 0b00001111),
            ((chunk[2] & 0b00001111) << 1) | ((chunk[3] >> 7) & 0b00000001),
            ((chunk[3] >> 2) & 0b00011111),
            ((chunk[3] << 3) & 0b00011000) | ((chunk[4] >> 5) & 0b00000111),
            (chunk[4] & 0b00011111),
        ];

        for byte in buffer {
            result.push(BASE32_ALPHABET[byte as usize] as char);
        }
    }

    let padding_len = content.len() * 8 % 5;
    if padding_len != 0 {
        let end = match padding_len {
            1 => "====",
            2 => "=",
            3 => "======",
            4 => "===",
            _ => "",
        };
        result.replace_range((result.len() - end.len())..result.len(), end);
    }

    result
}

/// # Функция нахождения порядкового номера символа
fn base64_symbol_to_index(c: char) -> Option<u8> {
    match c {
        'A'..='Z' => Some(c as u8 - b'A'),
        'a'..='z' => Some(c as u8 - b'a' + 26),
        '0'..='9' => Some(c as u8 - b'0' + 52),
        '+' => Some(62),
        '/' => Some(63),
        '=' => Some(0),
        _ => None,
    }
}

/// # Функция декодирования BASE64
/// Кодирования набора байт в строку.
///
/// ## Параметры
/// - `encoded` данные в закодированном виде.
/// - `buffer` буффер, в который будет записываться результат
pub fn base64_decode(encoded: &str, buffer: &mut Vec<u8>) {
    let padding_len = encoded.matches('=').count();
    for chunk in encoded.as_bytes().chunks(4) {
        let bytes = [
            base64_symbol_to_index(chunk[0] as char).unwrap(),
            base64_symbol_to_index(chunk[1] as char).unwrap(),
            base64_symbol_to_index(chunk[2] as char).unwrap(),
            base64_symbol_to_index(chunk[3] as char).unwrap(),
        ];

        let tmp_buff = [
            (bytes[0] << 2) | (bytes[1] >> 4),
            ((bytes[1] << 4) & 0b11110000) | ((bytes[2] >> 2) & 0b00001111),
            ((bytes[2] << 6) & 0b11000000) | bytes[3],
        ];

        for byte in tmp_buff {
            buffer.push(byte);
        }
    }

    if padding_len != 0 {
        buffer.truncate(buffer.len() - padding_len);
    }
}

/// # Функция нахождения порядкового номера символа
fn base32_symbol_to_index(c: char) -> Option<u8> {
    match c {
        'A'..='Z' => Some(c as u8 - b'A'),
        '2'..='7' => Some(c as u8 - b'2' + 26),
        '=' => Some(0),
        _ => None,
    }
}

pub fn base32_decode(encoded: &str, buffer: &mut Vec<u8>) {
    let padding_len = match encoded.matches('=').count() {
        1 => 1,
        3 => 2,
        4 => 3,
        6 => 4,
        _ => 0
    };

    for chunk in encoded.as_bytes().chunks(8) {
        let bytes = [
            base32_symbol_to_index(chunk[0] as char).unwrap(),
            base32_symbol_to_index(chunk[1] as char).unwrap(),
            base32_symbol_to_index(chunk[2] as char).unwrap(),
            base32_symbol_to_index(chunk[3] as char).unwrap(),
            base32_symbol_to_index(chunk[4] as char).unwrap(),
            base32_symbol_to_index(chunk[5] as char).unwrap(),
            base32_symbol_to_index(chunk[6] as char).unwrap(),
            base32_symbol_to_index(chunk[7] as char).unwrap(),
        ];

        let tmp_buff = [
            (bytes[0] << 3) | (bytes[1] >> 2),
            (bytes[1] << 6) | (bytes[2] << 1) | (bytes[3] >> 4),
            (bytes[3] << 4) | (bytes[4] >> 1),
            (bytes[4] << 7) | (bytes[5] << 2) | (bytes[6] >> 3),
            (bytes[6] << 5) | bytes[7],
        ];

        for byte in tmp_buff {
            buffer.push(byte);
        }
    }

    if padding_len != 0 {
        buffer.truncate(buffer.len() - padding_len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_decode_test() -> Result<(), String> {
        let test_cases = [
            ("bWVzc2FnZQ==", "message", 7),
            ("ZWNobw==", "echo", 4),
            ("Zm94", "fox", 3),
            ("ZGVhdGg=", "death", 5),
        ];

        for (input, expected, expected_len) in test_cases.iter() {
            let mut buffer: Vec<u8> = vec![];
            base64_decode(input, &mut buffer);
            
            assert_eq!(buffer.len(), *expected_len);

            let result = String::from_utf8(buffer).unwrap();
            assert_eq!(result, *expected);
        }

        Ok(())
    }

    #[test]
    fn base32_decode_test() -> Result<(), String> {
        let test_cases = [
            ("NVSXG43BM5SQ====", "message", 7),
            ("MVRWQ3Y=", "echo", 4),
            ("MZXXQ===", "fox", 3),
            ("MRSWC5DI", "death", 5),
            ("ONUWYZLOOQ======", "silent", 6),
        ];

        for (input, expected, expected_len) in test_cases.iter() {
            let mut buffer: Vec<u8> = vec![];
            base32_decode(input, &mut buffer);
            
            assert_eq!(buffer.len(), *expected_len);

            let result = String::from_utf8(buffer).unwrap();
            assert_eq!(result, *expected);
        }

        Ok(())
    }
}