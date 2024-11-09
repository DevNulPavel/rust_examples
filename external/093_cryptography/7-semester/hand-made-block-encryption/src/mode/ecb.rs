use crate::traits::{BlockCipher, Mode, CipherError};

#[derive(Debug, Clone)]
pub struct ECB;

impl Mode for ECB {
    fn encrypt(&self, cipher: &dyn BlockCipher, data: &[u8]) -> Result<Vec<u8>, CipherError> {
        let block_size = cipher.block_size();

        if data.is_empty() {
            return Err(CipherError::DataTooShort);
        }

        if data.len() % block_size != 0 {
            return Err(CipherError::DataNotAligned);
        }

        let mut result = Vec::with_capacity(data.len());

        for chunk in data.chunks(block_size) {
            let encrypted_block = cipher.encrypt_block(chunk)?;
            result.extend_from_slice(&encrypted_block);
        }

        Ok(result)
    }

    fn decrypt(&self, cipher: &dyn BlockCipher, data: &[u8]) -> Result<Vec<u8>, CipherError> {
        let block_size = cipher.block_size();

        if data.is_empty() {
            return Err(CipherError::DataTooShort);
        }

        if data.len() % block_size != 0 {
            return Err(CipherError::DataNotAligned);
        }

        let mut result = Vec::with_capacity(data.len());

        for chunk in data.chunks(block_size) {
            let decrypted_block = cipher.decrypt_block(chunk)?;
            result.extend_from_slice(&decrypted_block);
        }

        Ok(result)
    }

    fn is_block_mode(&self) -> bool {
        true
    }
}
