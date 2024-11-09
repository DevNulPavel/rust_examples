use crate::traits::{BlockCipher, CipherError};

/// Длина блока: 64 бита
const BLOCK_SIZE: usize = 8;
/// Длина ключа: 256 бит
const KEY_SIZE: usize = 32;
/// Количество раундов шифрования: 32
const ROUNDS: usize = 32;

/// # Шифр "Магма"
///
/// - Длина блока: 64 бита
/// - Длина ключа: 256 бит
/// - Количество раундов шифрования: 32
#[derive(Debug, Clone)]
pub struct Magma {
    round_keys: [u32; ROUNDS],
    matrix_pi: [[u8; 16]; 8],
}

impl Magma {
    pub fn new(key: &[u8], matrix_pi: [[u8; 16]; 8]) -> Result<Self, CipherError> {
        // Недопускается ключ неверной длины
        if key.len() != KEY_SIZE {
            return Err(CipherError::InvalidKeyLenght);
        }

        // Недопускается ключ, состоящий из нулей
        if key.iter().all(|&x| x == 0) {
            return Err(CipherError::InvalidKeyFormat);
        }

        // Первые 24 раундовых ключа
        // с порядком следования 0, 1, 2, 3, 4, 5, 6, 7
        let mut round_keys = [0u32; ROUNDS];
        for i in 0..3 {
            for (j, chunk) in key.chunks_exact(4).enumerate() {
                let key = u32::from_le_bytes([chunk[3], chunk[2], chunk[1], chunk[0]]);
                round_keys[j + 8 * i] = key;
            }
        }

        // Последние 8 раундовых ключа
        // с порядком следования 7, 6, 5, 4, 3, 2, 1, 0
        for (j, chunk) in key.chunks_exact(4).rev().enumerate() {
            let key = u32::from_le_bytes([chunk[3], chunk[2], chunk[1], chunk[0]]);
            round_keys[j + 24] = key;
        }

        Ok(Self {
            round_keys,
            matrix_pi,
        })
    }

    pub fn get_round_keys(&self) -> [u32; ROUNDS] {
        self.round_keys
    }

    fn transform_t(&self, value: u32) -> u32 {
        let mut result: u32 = 0;

        for (i, row) in self.matrix_pi.iter().enumerate() {
            let part = (value >> (4 * i)) & 0x0f;

            let substituted = row[part as usize] as u32;

            result |= substituted << (4 * i);
        }

        result
    }

    fn transform_g(&self, a: u32, k: u32) -> u32 {
        let temp = a.wrapping_add(k);
        let substituted = self.transform_t(temp);
        substituted.rotate_left(11)
    }

    #[cfg(test)]
    pub(crate) fn test_transform_t(&self, value: u32) -> u32 {
        self.transform_t(value)
    }

    #[cfg(test)]
    pub(crate) fn test_transform_g(&self, a: u32, k: u32) -> u32 {
        self.transform_g(a, k)
    }

    #[cfg(test)]
    pub(crate) fn test_encrypt_block(&self, block: &[u8]) -> Result<Vec<u8>, CipherError> {
        self.encrypt_block(block)
    }

    #[cfg(test)]
    pub(crate) fn test_decrypt_block(&self, block: &[u8]) -> Result<Vec<u8>, CipherError> {
        self.decrypt_block(block)
    }
}

impl BlockCipher for Magma {
    fn block_size(&self) -> usize {
        BLOCK_SIZE
    }

    fn encrypt_block(&self, block: &[u8]) -> Result<Vec<u8>, CipherError> {
        if block.len() != BLOCK_SIZE {
            return Err(CipherError::InvalidBlockSize);
        }

        if block.is_empty() {
            return Err(CipherError::DataTooShort);
        }

        let mut a0 = u32::from_le_bytes([block[3], block[2], block[1], block[0]]);
        let mut a1 = u32::from_le_bytes([block[7], block[6], block[5], block[4]]);

        // 31 раунд шифрования
        for i in 0..31 {
            let temp = a1;
            a1 = a0 ^ self.transform_g(a1, self.round_keys[i]);
            a0 = temp;
        }
        // 32-й раунд
        let temp = a1;
        a1 = a0 ^ self.transform_g(a1, self.round_keys[31]);
        a0 = temp;

        let mut result = Vec::with_capacity(BLOCK_SIZE);
        result.extend_from_slice(&a1.to_le_bytes());
        result.extend_from_slice(&a0.to_le_bytes());

        result[..4].reverse();
        result[4..].reverse();

        Ok(result)
    }

    fn decrypt_block(&self, block: &[u8]) -> Result<Vec<u8>, CipherError> {
        if block.len() != BLOCK_SIZE {
            return Err(CipherError::InvalidBlockSize);
        }

        if block.is_empty() {
            return Err(CipherError::DataTooShort);
        }

        let mut a0 = u32::from_le_bytes([block[3], block[2], block[1], block[0]]);
        let mut a1 = u32::from_le_bytes([block[7], block[6], block[5], block[4]]);


        // Первый раунд расшифрования
        let temp = a1;
        a1 = a0 ^ self.transform_g(a1, self.round_keys[31]);
        a0 = temp;

        // Остальные 31 раунды расшифрования
        for i in (0..31).rev() {
            let temp = a1;
            a1 = a0 ^ self.transform_g(a1, self.round_keys[i]);
            a0 = temp;
        }

        let mut result = Vec::with_capacity(BLOCK_SIZE);
        result.extend_from_slice(&a1.to_le_bytes());
        result.extend_from_slice(&a0.to_le_bytes());

        result[..4].reverse();
        result[4..].reverse();

        Ok(result)
    }
}
