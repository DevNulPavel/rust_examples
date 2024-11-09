use crate::traits::{BlockCipher, Mode, CipherError};

#[derive(Debug, Clone)]
pub struct CBC {
    iv: Vec<u8>,
}

impl CBC {
    pub fn new(iv: Vec<u8>) -> Self {
        Self { iv }
    }
}

impl Mode for CBC {
    fn encrypt(&self, cipher: &dyn BlockCipher, data: &[u8]) -> Result<Vec<u8>, CipherError> {
        let block_size = cipher.block_size();

        if data.is_empty() {
            return Err(CipherError::DataTooShort);
        }

        if data.len() % block_size != 0 {
            return Err(CipherError::DataNotAligned);
        }

        if self.iv.len() != block_size {
            return Err(CipherError::InvalidIVLenght);
        }

        let mut result = Vec::with_capacity(data.len());
        let mut prev_block = self.iv.clone();

        for chunk in data.chunks(block_size) {
            let mut block = chunk.to_vec();

            for i in 0..block_size {
                block[i] ^= prev_block[i];
            }

            let encrypted_block = cipher.encrypt_block(&block)?;
            result.extend_from_slice(&encrypted_block);
            prev_block = encrypted_block;
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

        if self.iv.len() != block_size {
            return Err(CipherError::InvalidIVLenght);
        }

        let mut result = Vec::with_capacity(data.len());
        let mut prev_block = self.iv.clone();

        for chunk in data.chunks(block_size) {
            let mut decrypted_block = cipher.decrypt_block(chunk)?;

            for i in 0..block_size {
                decrypted_block[i] ^= prev_block[i];
            }

            result.extend_from_slice(&decrypted_block);
            prev_block = chunk.to_vec();
        }

        Ok(result)
    }

    fn is_block_mode(&self) -> bool {
        true
    }
}
