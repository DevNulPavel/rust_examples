use std::iter::repeat;

use crate::traits::{CipherError, Padding};

/// Максимальная длина блока: 256 байт
const MAX_BLOCK_SIZE: usize = 256;

#[derive(Debug, Clone)]
pub struct PKCS7;

#[cfg(test)]
impl PKCS7 {
    pub(crate) fn test_pad(&self, data: &[u8], block_size: usize) -> Result<Vec<u8>, CipherError> {
        self.pad(data, block_size)
    }

    pub(crate) fn test_unpad(&self, data: &[u8]) -> Result<Vec<u8>, CipherError> {
        self.unpad(data)
    }
}


impl Padding for PKCS7 {
    fn pad(&self, data: &[u8], block_size: usize) -> Result<Vec<u8>, CipherError> {
        if block_size == 0 {
            return Err(CipherError::InvalidBlockSize);
        }

        if block_size >= MAX_BLOCK_SIZE {
            return Err(CipherError::InvalidBlockSize);
        }

        let padding_len = block_size - (data.len() % block_size);
        let mut padded = Vec::with_capacity(data.len() + padding_len);
        padded.extend_from_slice(data);
        padded.extend(repeat(padding_len as u8).take(padding_len));

        Ok(padded)
    }

    fn unpad(&self, data: &[u8]) -> Result<Vec<u8>, CipherError> {
        if data.is_empty() {
            return Err(CipherError::DataTooShort);
        }

        // Проверка длины заполнения
        // Заполнение не должно быть нулевым и
        // не должно быть больше самих данных
        let padding_len = data[data.len() - 1] as usize;
        if padding_len == 0 || padding_len > data.len(){
            return Err(CipherError::InvalidPadding);
        }

        // Проверка байтов заполнения
        // Все байты заполнения должны быть одинаковыми
        // и равными длине заполнения
        for &byte in &data[data.len() - padding_len..] {
            if byte != padding_len as u8 {
                return Err(CipherError::InvalidPadding);
            }
        }

        Ok(data[..data.len() - padding_len].to_vec())
    }
}
