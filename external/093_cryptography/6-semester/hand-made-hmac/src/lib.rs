use hand_made_sha::{sha256, sha512};

const IPAD_BYTE: u8 = 0x36;
const OPAD_BYTE: u8 = 0x5c;

fn slice_xor(input1: &[u8], input2: &[u8], result: &mut [u8]) {
    if input1.len() != input2.len() {
        panic!()
    }
    if input1.len() != result.len() {
        panic!()
    }
    for i in 0..input1.len() {
        result[i] = input1[i] ^ input2[i];
    }
}

trait HmacHashFunction {
    /// # Рассчет хэша
    fn get_hash(&self, message: &[u8]) -> Vec<u8>;
    /// # Получение размера входного блока хэш-функции
    fn get_block_size(&self) -> u32;
    /// # Получение размера выходного хэша
    fn get_hash_size(&self) -> u32;
}
enum HashFunciton {
    Sha256,
    Sha512,
    Streebog256,
    Streebog512,
}

impl HmacHashFunction for HashFunciton {
    fn get_hash(&self, message: &[u8]) -> Vec<u8> {
        match self {
            HashFunciton::Sha256 => {
                sha256(message).to_vec()
            },
            HashFunciton::Sha512 => {
                sha512(message).to_vec()
            },
            HashFunciton::Streebog256 => {
                todo!()
            },
            HashFunciton::Streebog512 => {
                todo!()
            },
        }
    }

    fn get_block_size(&self) -> u32 {
        match self {
            HashFunciton::Sha256 => {
                64
            },
            HashFunciton::Sha512 => {
                128
            },
            HashFunciton::Streebog256 => {
                32
            },
            HashFunciton::Streebog512 => {
                64
            },
        }
    }

    fn get_hash_size(&self) -> u32 {
        match self {
            HashFunciton::Sha256 => {
                32
            },
            HashFunciton::Sha512 => {
                64
            },
            HashFunciton::Streebog256 => {
                32
            },
            HashFunciton::Streebog512 => {
                64
            },
        }
    }
}

pub fn hmac_sha256(message: &[u8], key: &[u8]) -> [u8; 32] {
    const BLOCK_SIZE: usize = 64;

    // Получение подходящего по длине ключа K0
    let mut k0 = [0u8; BLOCK_SIZE];
    if key.len() == BLOCK_SIZE {
        k0 = key.try_into().unwrap();
    } else if key.len() > BLOCK_SIZE {
        k0[..32].copy_from_slice(&sha256(key));
    } else {
        k0[..key.len()].copy_from_slice(&key);
    }

    // Подготовка строк Si и So
    let mut si = [0u8; BLOCK_SIZE];
    slice_xor(&k0, &[IPAD_BYTE; BLOCK_SIZE], &mut si);
    let mut so = [0u8; BLOCK_SIZE];
    slice_xor(&k0, &[OPAD_BYTE; BLOCK_SIZE], &mut so);

    let mut first_part: Vec<u8> = Vec::with_capacity(BLOCK_SIZE + message.len());

    first_part.extend_from_slice(&si);
    first_part.extend_from_slice(message);

    let first_hash = sha256(&first_part);

    let mut second_part = Vec::with_capacity(BLOCK_SIZE + first_hash.len());

    second_part.extend_from_slice(&so);
    second_part.extend_from_slice(&first_hash);

    sha256(&second_part)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use super::*;

    #[test]
    fn hmac_sha256_text_test() -> Result<(), String> {
        let test_cases = [
            (
                "hello world",
                "secret key",
                [
                    0xc6, 0x1b, 0x51, 0x98,
                    0xdf, 0x58, 0x63, 0x9e,
                    0xdb, 0x98, 0x92, 0x51,
                    0x47, 0x56, 0xb8, 0x9a,
                    0x36, 0x85, 0x6d, 0x82,
                    0x6e, 0x5d, 0x85, 0x02,
                    0x3a, 0xb1, 0x81, 0xb4,
                    0x8e, 0xa5, 0xd0, 0x18,
                ]
            ),
            (
                "привет мир",
                "secret key",
                [
                    0xc8, 0x30, 0xb7, 0x18,
                    0x26, 0xa6, 0x9f, 0x77,
                    0x2c, 0xd1, 0x31, 0x69,
                    0x0d, 0x4f, 0xc0, 0x60,
                    0xa5, 0xb8, 0xd1, 0xf5,
                    0x8a, 0xa2, 0x87, 0xe4,
                    0x64, 0xec, 0x26, 0x0a,
                    0xd8, 0x81, 0x7e, 0x1e,
                ]
            ),
            (
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.",
                "secret key",
                [
                    0x43, 0xd5, 0xf0, 0x9c,
                    0xbc, 0xdf, 0xc6, 0x6f,
                    0xec, 0xc5, 0x1f, 0x81,
                    0x63, 0x7e, 0x65, 0xcc,
                    0x48, 0xd4, 0xd4, 0xad,
                    0xf3, 0x9b, 0x3e, 0x86,
                    0xe4, 0x18, 0x2e, 0x51,
                    0xb7, 0x2e, 0xdb, 0xbd,
                ]
            ),
        ];

        for (msg, key, expected) in test_cases {
            let hash = hmac_sha256(msg.as_bytes(), key.as_bytes());
            assert_eq!(hash, expected);
        }

        Ok(())
    }

    #[test]
    fn hmac_sha256_video_test() -> Result<(), String> {
        let test_cases = [
            (
                Path::new("./assets/video.mp4"),
                "secret key",
                [
                    0xb6, 0xa5, 0x63, 0xfa,
                    0x06, 0x3f, 0x5e, 0x0d,
                    0x0e, 0x11, 0xe5, 0x6b,
                    0xaf, 0xe7, 0xe9, 0x07,
                    0x15, 0xc6, 0xec, 0x07,
                    0x46, 0xfd, 0xfd, 0xeb,
                    0x15, 0xba, 0x54, 0xfc,
                    0xf4, 0x71, 0x1f, 0x15,
                ]
            ),
        ];

        for (path, key, expected) in test_cases {
            let msg = fs::read(path).expect("Ошибка чтений файла");
            let hash = hmac_sha256(msg.as_slice(), key.as_bytes());
            assert_eq!(hash, expected);
        }

        Ok(())
    }
}