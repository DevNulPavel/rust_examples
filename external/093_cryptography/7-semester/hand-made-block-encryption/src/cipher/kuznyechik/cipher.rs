use crate::traits::{BlockCipher, CipherError};

use super::constants::{PI, PI_REV, L};

/// Длина блока: 128 бит
const BLOCK_SIZE: usize = 16;
/// Длина ключа: 256 бит
const KEY_SIZE: usize = 32;
/// Количество раундов шифрования: 10
const ROUNDS: usize = 10;

/// # Шифр "Кузнечик"
///
/// - Длина блока: 128 бит
/// - Длина ключа: 256 бит
/// - Количество раундов шифрования: 10
#[derive(Debug, Clone)]
pub struct Kuznyechik {
    round_keys: [[u8; 16]; ROUNDS],
}

impl Kuznyechik {
    pub fn new(key: &[u8]) -> Result<Self, CipherError> {
        // Недопускается ключ неверной длины
        if key.len() != KEY_SIZE {
            return Err(CipherError::InvalidKeyLenght);
        }

        // Недопускается ключ, состоящий из нулей
        if key.iter().all(|&x| x == 0) {
            return Err(CipherError::InvalidKeyFormat);
        }

        // Формирование итерационных констант
        let mut iter_constants = [[0u8; 16]; 32];
        for (i, e) in iter_constants.iter_mut().enumerate() {
            e[15] = i as u8 + 1;
            Self::transform_l(e);
        }

        let mut round_keys = [[0u8; 16]; ROUNDS];
        let mut temp_1 = [0u8; 16];
        let mut temp_2 = [0u8; 16];
        let mut temp_3 = [0u8; 16];
        let mut temp_4 = [0u8; 16];

        round_keys[0].copy_from_slice(&key[..16]);
        round_keys[1].copy_from_slice(&key[16..]);
        temp_1.copy_from_slice(&round_keys[0]);
        temp_2.copy_from_slice(&round_keys[1]);

        for i in 0..4 {
            Self::transform_f(&mut temp_1, &mut temp_2, &mut temp_3, &mut temp_4, &mut iter_constants[8 * i]);
            Self::transform_f(&mut temp_3, &mut temp_4, &mut temp_1, &mut temp_2, &mut iter_constants[1 + 8 * i]);
            Self::transform_f(&mut temp_1, &mut temp_2, &mut temp_3, &mut temp_4, &mut iter_constants[2 + 8 * i]);
            Self::transform_f(&mut temp_3, &mut temp_4, &mut temp_1, &mut temp_2, &mut iter_constants[3 + 8 * i]);
            Self::transform_f(&mut temp_1, &mut temp_2, &mut temp_3, &mut temp_4, &mut iter_constants[4 + 8 * i]);
            Self::transform_f(&mut temp_3, &mut temp_4, &mut temp_1, &mut temp_2, &mut iter_constants[5 + 8 * i]);
            Self::transform_f(&mut temp_1, &mut temp_2, &mut temp_3, &mut temp_4, &mut iter_constants[6 + 8 * i]);
            Self::transform_f(&mut temp_3, &mut temp_4, &mut temp_1, &mut temp_2, &mut iter_constants[7 + 8 * i]);
            round_keys[2 + 2 * i].copy_from_slice(&temp_1);
            round_keys[3 + 2 * i].copy_from_slice(&temp_2);
        }

        Ok(Self { round_keys })
    }

    /// # Получение раундовых ключей
    pub fn get_round_keys(&self) -> [[u8; 16]; 10] {
        self.round_keys
    }

    /// # Преобразование X
    fn transform_x(a: &[u8; 16], b: &[u8; 16], result: &mut [u8; 16]) {
        for (i, e) in result.iter_mut().enumerate() {
            *e = a[i] ^ b[i];
        }
    }

    /// # Преобразование S
    fn transform_s(result: &mut [u8; 16]) {
        result.iter_mut().for_each(|byte| *byte = PI[*byte as usize]);
    }

    /// # Преобразование обратное преобразованию S
    fn transform_s_rev(result: &mut [u8; 16]) {
        result.iter_mut().for_each(|byte| *byte = PI_REV[*byte as usize]);
    }

    /// # Умножение в поле Галуа
    ///
    /// Умножение в поле Галуа, построенном над
    /// неприводимым полиномом
    /// `x^8 + x^7 + x^6 + x + 1`
    fn gf_multiply(mut a: u8, mut b: u8) -> u8 {
        let mut result: u8 = 0;
        let mut high_bit: u8;

        for _ in 0..8 {
            if b & 0b00000001 == 0b00000001 {
                result ^= a;
            }

            high_bit = a & 0b10000000;
            a <<= 1;

            if high_bit == 0b10000000 {
                a ^= 0b11000011;
            }

            b >>= 1;
        }

        result
    }

    /// # Преобразование R
    fn transform_r(result: &mut [u8; 16]) {
        let mut temp = 0;
        for (i, e) in result.iter().enumerate() {
            temp ^= Self::gf_multiply(*e, L[15 - i]);
        }

        result.rotate_right(1);

        result[0] = temp;
    }

    /// # Преобразование обратное преобразованию R
    fn transform_r_rev(result: &mut [u8; 16]) {
        let mut temp = result[0];

        result.rotate_left(1);

        for (i, e) in result[..15].iter().enumerate() {
            temp ^= Self::gf_multiply(*e, L[15 - i]);
        }

        result[15] = temp;
    }

    /// # Преобразование L
    fn transform_l(result: &mut [u8; 16]) {
        for _ in 0..16 {
            Self::transform_r(result);
        }
    }

    /// # Преобразование обратное преобразованию L
    fn transform_l_rev(result: &mut [u8; 16]) {
        for _ in 0..16 {
            Self::transform_r_rev(result);
        }
    }

    /// # Итерация развертывания ключа
    fn transform_f(
        in_key_1: &mut [u8; 16],
        in_key_2: &mut [u8; 16],
        out_key_1: &mut [u8; 16],
        out_key_2: &mut [u8; 16],
        iter_constant: &mut [u8; 16],
    ) {
        let mut temp = [0u8; 16];

        out_key_2.copy_from_slice(in_key_1);

        Self::transform_x(in_key_1, iter_constant, &mut temp);
        Self::transform_s(&mut temp);
        Self::transform_l(&mut temp);
        Self::transform_x(&temp, in_key_2, out_key_1);
    }

    #[cfg(test)]
    pub(crate) fn test_transform_s(result: &mut [u8; 16]) {
        Self::transform_s(result)
    }

    #[cfg(test)]
    pub(crate) fn test_transform_s_rev(result: &mut [u8; 16]) {
        Self::transform_s_rev(result)
    }

    #[cfg(test)]
    pub(crate) fn test_gf_multiply(a: u8, b: u8) -> u8 {
        Self::gf_multiply(a, b)
    }

    #[cfg(test)]
    pub(crate) fn test_transform_r(result: &mut [u8; 16]) {
        Self::transform_r(result)
    }

    #[cfg(test)]
    pub(crate) fn test_transform_r_rev(result: &mut [u8; 16]) {
        Self::transform_r_rev(result)
    }

    #[cfg(test)]
    pub(crate) fn test_transform_l(result: &mut [u8; 16]) {
        Self::transform_l(result)
    }

    #[cfg(test)]
    pub(crate) fn test_transform_l_rev(result: &mut [u8; 16]) {
        Self::transform_l_rev(result)
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

impl BlockCipher for Kuznyechik {
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

        let mut block_exact: [u8; 16] = [0u8; 16];
        block_exact.copy_from_slice(block);

        for key in self.round_keys[..9].iter() {
            Self::transform_x(key, &block_exact.clone(), &mut block_exact);
            Self::transform_s(&mut block_exact);
            Self::transform_l(&mut block_exact);
        }
        Self::transform_x(&self.round_keys[9], &block_exact.clone(), &mut block_exact);

        Ok(block_exact.to_vec())
    }

    fn decrypt_block(&self, block: &[u8]) -> Result<Vec<u8>, CipherError> {
        if block.len() != BLOCK_SIZE {
            return Err(CipherError::InvalidBlockSize);
        }

        if block.is_empty() {
            return Err(CipherError::DataTooShort);
        }

        let mut block_exact: [u8; 16] = [0u8; 16];
        block_exact.copy_from_slice(block);

        Self::transform_x(&self.round_keys[9], &block_exact.clone(), &mut block_exact);
        for key in self.round_keys[..9].iter().rev() {
            Self::transform_l_rev(&mut block_exact);
            Self::transform_s_rev(&mut block_exact);
            Self::transform_x(key, &block_exact.clone(), &mut block_exact);
        }

        Ok(block_exact.to_vec())
    }
}
