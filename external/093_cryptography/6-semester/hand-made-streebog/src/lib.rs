mod constants;

use constants::{PI, TAU, A, C};

/// # Сложение двух 512 битных чисел
fn add_512(a: &[u8; 64], b: &[u8; 64], result: &mut [u8; 64]) {
    let mut temp: u16 = 0;
    for i in (0..64).rev() {
        temp = a[i] as u16 + b[i] as u16 + (temp >> 8);
        result[i] = temp as u8;
    }
}

/// # X-преобразование
/// На вход функции X подаются две последовательности длиной 512 бит каждая,
/// выходом функции является XOR этих двух последовательностей.
fn transform_x(a: &[u8; 64], b: &[u8; 64], result: &mut [u8; 64]) {
    for (index, byte) in result.iter_mut().enumerate() {
        *byte = a[index] ^ b[index];
    }
}

/// # S-преобразование
/// Функция S является обычной функцией подстановки. Каждый байт из 512-битной
/// входной последовательности заменяется соответствующим байтом из таблицы
/// подстановок PI.
fn transform_s(result: &mut [u8; 64]) {
    result.iter_mut().for_each(|byte| *byte = PI[*byte as usize]);
}

/// # P-преобразование
/// Функция перестановки. Для каждой пары байт из входной последовательности
/// происходит замена одного байта другим.
fn transform_p(result: &mut [u8; 64]) {
    let temp = result.clone();
    for (index, position) in TAU.iter().enumerate() {
        result[index] = temp[*position as usize];
    }
}

/// # L-преобразование
/// Представляет собой умножение 64-битного входного вектора на
/// бинарную матрицу А размерами 64 на 64.
fn transform_l(result: &mut [u8; 64]) {
    // Объединение 8 байт в одну последовательность
    let input_u64: [u64; 8] = result.chunks_exact(8)
        .map(|bytes| u64::from_be_bytes(bytes.try_into().unwrap()))
        .collect::<Vec<u64>>()
        .try_into()
        .unwrap();

    let mut buffers = [0u64; 8];

    for i in 0..8 {
        for j in 0..64 {
            if (input_u64[i] >> j) & 1 == 1 {
                buffers[i] ^= A[63 - j];
            }
        }
    }

    // Разделение модифицированной последовательности на части по 8 байт
    let buffer: [u8; 64] = buffers.iter()
        .flat_map(|bytes| bytes.to_be_bytes())
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();

    for (index, byte) in buffer.iter().enumerate() {
        result[index] = *byte;
    }
}

/// # XSPL-шифр
/// Функция вычисления раундовых ключей.
/// 1. X - Сложение по модулю 2.
/// 2. S - Подстановка.
/// 3. P - Перестановка.
/// 4. L - Линейное преобразование.
fn key_schedule(keys: &mut [u8; 64], iter_index: usize) {
    transform_x(&keys.clone(), &C[iter_index], keys);
    transform_s(keys);
    transform_p(keys);
    transform_l(keys);
}

/// # E-преобразование
/// Является частью функции сжатия.
fn transform_e(
    keys: &mut [u8; 64],
    block: &[u8; 64],
    state: &mut [u8; 64]
) {
    transform_x(block, keys, state);

    for i in 0..12 {
        transform_s(state);
        transform_p(state);
        transform_l(state);
        key_schedule(keys, i);
        transform_x(&state.clone(), keys, state);
    }
}

/// # Функция сжатия g
fn transform_g(
    n: &[u8; 64],
    hash: &mut [u8; 64],
    message: &[u8; 64],
) {
    let mut keys = [0u8; 64];
    let mut temp = [0u8; 64];

    transform_x(n, hash, &mut keys);

    transform_s(&mut keys);
    transform_p(&mut keys);
    transform_l(&mut keys);

    transform_e(&mut keys, message, &mut temp);

    transform_x(&temp.clone(), hash, &mut temp);
    transform_x(&temp, message, hash);
}

/// # Общая часть для обеих версий ГОСТ Р 34.11-2012
fn streebog_core(message: &[u8], hash: &mut [u8; 64]) {
    let mut n = [0u8; 64];
    let mut sigma = [0u8; 64];

    // Массив, хранящий размер полезных данных блока
    let mut block_size = [0u8; 64];
    let mut block: [u8; 64];

    for chunk in message.chunks(64) {
        // Вычисление размера полезных данных блока
        let chunk_size = chunk.len() as u16 * 8;
        block_size[62..].copy_from_slice(&chunk_size.to_be_bytes());

        // Дополнение блока
        if chunk.len() != 64 {
            block = [0u8; 64];
            block[..chunk.len()].copy_from_slice(chunk);
            block[chunk.len()] = 1;
        } else {
            block = chunk.try_into().unwrap();
        }
        block.reverse();

        transform_g(&n, hash, &block);
        add_512(&n.clone(), &block_size, &mut n);
        add_512(&sigma.clone(), &block, &mut sigma);
    }

    transform_g(&[0u8; 64], hash, &n);
    transform_g(&[0u8; 64], hash, &sigma);

    hash.reverse();
}

/// # ГОСТ Р 34.11-2012 512 бит
pub fn streebog_512(message: &[u8]) -> [u8; 64] {
    let mut hash = [0u8; 64];

    streebog_core(message, &mut hash);

    hash
}

/// # ГОСТ Р 34.11-2012 256 бит
pub fn streebog_256(message: &[u8]) -> [u8; 32] {
    let mut hash = [1u8; 64];

    streebog_core(message, &mut hash);

    hash[32..].try_into().unwrap()
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path};

    #[test]
    fn streebog_256_text_test() -> Result<(), String> {
        let test_cases = [
            (
                "hello world\n",
                [
                    0xf7, 0x20, 0x18, 0x18, 0x9a, 0x5c, 0xfb, 0x80,
                    0x3d, 0xbe, 0x1f, 0x21, 0x49, 0xcf, 0x55, 0x4c,
                    0x40, 0x09, 0x3d, 0x8e, 0x7f, 0x81, 0xc2, 0x1e,
                    0x08, 0xac, 0x5b, 0xcd, 0x09, 0xd9, 0x93, 0x4d,
                ]
            ),
            (
                "привет мир\n",
                [
                    0xa0, 0x37, 0x66, 0x66, 0xdb, 0x84, 0x45, 0x55,
                    0xaa, 0x12, 0xda, 0xa0, 0x35, 0x09, 0xb5, 0xd6,
                    0x7f, 0xf4, 0x74, 0x19, 0x9b, 0xe6, 0xbc, 0x33,
                    0xc7, 0xde, 0xcb, 0xb9, 0xf8, 0xfb, 0xc3, 0x2d,
                ]
            ),
            (
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.\n",
                [
                    0x3e, 0x8e, 0x39, 0x1b, 0xbc, 0x40, 0xe3, 0x60,
                    0x0f, 0x87, 0xdd, 0xcb, 0x27, 0xeb, 0x7a, 0x83,
                    0x91, 0x89, 0x56, 0x7c, 0x5e, 0xd4, 0xfa, 0x6f,
                    0xe4, 0x34, 0x1b, 0x42, 0x4e, 0x77, 0x01, 0xb1,
                ]
            ),
        ];

        for (input, expected) in test_cases {
            let hash = streebog_256(input.as_bytes());
            assert_eq!(hash, expected);
        }

        Ok(())
    }

    #[test]
    fn streebog_256_video_test() -> Result<(), String> {
        let test_cases = [
            (
                Path::new("./assets/video.mp4"),
                [
                    0x0f, 0xf9, 0x7f, 0xe9, 0x46, 0xed, 0xcf, 0x68,
                    0x64, 0x30, 0x50, 0xe0, 0xd0, 0xc8, 0x60, 0xcd,
                    0xd4, 0x39, 0xfa, 0xef, 0x8f, 0x1b, 0x20, 0xe1,
                    0xd0, 0x44, 0x2b, 0xa4, 0x65, 0x55, 0xfa, 0x17,
                ]
            ),
        ];

        for (input, expected) in test_cases {
            let data = fs::read(input).expect("Ошибка чтений файла");
            let hash = streebog_256(data.as_slice());
            assert_eq!(hash, expected);
        }

        Ok(())
    }

    #[test]
    fn streebog_512_text_test() -> Result<(), String> {
        let test_cases = [
            (
                "hello world\n",
                [
                    0x9d, 0x29, 0x5f, 0xa5, 0x6e, 0xbe, 0x77, 0xb8,
                    0x3d, 0xb3, 0x78, 0x32, 0x68, 0x5c, 0xe8, 0x74,
                    0xc4, 0x3a, 0x5a, 0xdd, 0x7a, 0xfc, 0x5f, 0x1a,
                    0xaa, 0x94, 0xca, 0x21, 0xb1, 0x2a, 0x12, 0x89,
                    0x7a, 0x48, 0xbb, 0x3d, 0xbb, 0xe2, 0x0c, 0xd9,
                    0xcf, 0xaf, 0xa2, 0x2a, 0x6e, 0x3c, 0x82, 0xeb,
                    0x4c, 0x65, 0x03, 0x10, 0x9b, 0xfb, 0x0b, 0x45,
                    0x14, 0xc7, 0xbc, 0x27, 0xe6, 0x9e, 0xc1, 0x20,
                ]
            ),
            (
                "привет мир\n",
                [
                    0xb6, 0xef, 0x67, 0x2f, 0xd3, 0x47, 0x21, 0x26,
                    0xa3, 0xb6, 0xec, 0x6b, 0xe9, 0xe4, 0x45, 0xbb,
                    0x66, 0xd1, 0xa5, 0x33, 0x61, 0x19, 0x6b, 0xfb,
                    0x16, 0xa8, 0x5f, 0xf2, 0xf9, 0x46, 0x9d, 0x8e,
                    0xa5, 0x18, 0x2d, 0x6a, 0x24, 0x60, 0x1f, 0x25,
                    0x18, 0xb9, 0x17, 0x6d, 0x57, 0xbe, 0x34, 0x62,
                    0x06, 0x3e, 0x2d, 0x90, 0xab, 0x4e, 0xc1, 0xa6,
                    0xa6, 0xd9, 0x2a, 0xaf, 0x2d, 0xb7, 0x73, 0xa2,
                ]
            ),
            (
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.\n",
                [
                    0x81, 0xb2, 0x34, 0x59, 0x36, 0x56, 0xf0, 0x4e,
                    0xe1, 0xbc, 0x47, 0x1a, 0xf2, 0xcc, 0x26, 0x98,
                    0x95, 0xc7, 0xad, 0xed, 0x3c, 0x8d, 0x5d, 0x81,
                    0x8a, 0x18, 0xc5, 0x37, 0x89, 0x4b, 0xd6, 0x1c,
                    0x2d, 0xb1, 0xa4, 0xe6, 0x6f, 0xf0, 0x0d, 0x33,
                    0xfb, 0x7d, 0x2d, 0x19, 0x52, 0x02, 0x98, 0x4c,
                    0xad, 0xe7, 0x6b, 0x7c, 0x0a, 0xb3, 0xde, 0x8b,
                    0x9d, 0x61, 0xa2, 0x54, 0x1c, 0x04, 0x05, 0x91,
                ]
            ),
        ];

        for (input, expected) in test_cases {
            let hash = streebog_512(input.as_bytes());
            assert_eq!(hash, expected);
        }

        Ok(())
    }

    #[test]
    fn streebog_512_video_test() -> Result<(), String> {
        let test_cases = [
            (
                Path::new("./assets/video.mp4"),
                [
                    0xad, 0x22, 0x66, 0xff, 0x8c, 0x34, 0xd0, 0x04,
                    0x14, 0xea, 0x60, 0x4c, 0x3f, 0xd4, 0xbc, 0x1c,
                    0x49, 0x8a, 0xba, 0x09, 0xd5, 0x82, 0x18, 0x28,
                    0x0c, 0xe0, 0xbf, 0x0b, 0xec, 0xb4, 0xbf, 0xf5,
                    0x2a, 0x57, 0xee, 0x60, 0x29, 0xc5, 0x6a, 0x8c,
                    0x42, 0x69, 0x9a, 0xda, 0x45, 0xe6, 0xd9, 0x88,
                    0x8f, 0xad, 0xc4, 0x7d, 0xc3, 0x21, 0x4d, 0xef,
                    0xfd, 0xd8, 0x20, 0x69, 0x21, 0x2e, 0xe3, 0x74,
                ]
            ),
        ];

        for (input, expected) in test_cases {
            let data = fs::read(input).expect("Ошибка чтений файла");
            let hash = streebog_512(data.as_slice());
            assert_eq!(hash, expected);
        }

        Ok(())
    }
}
