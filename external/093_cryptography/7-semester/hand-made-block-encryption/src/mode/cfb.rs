use crate::traits::{BlockCipher, Mode, CipherError};

#[derive(Debug, Clone)]
pub struct CFB {
    iv: Vec<u8>,
}

impl CFB {
    pub fn new(iv: Vec<u8>) -> Self {
        Self { iv }
    }
}

impl Mode for CFB {
    fn encrypt(&self, cipher: &dyn BlockCipher, data: &[u8]) -> Result<Vec<u8>, CipherError> {
        let block_size = cipher.block_size();

        if data.is_empty() {
            return Err(CipherError::DataTooShort);
        }

        if self.iv.len() != block_size {
            return Err(CipherError::InvalidIVLenght);
        }

        let mut result = Vec::with_capacity(data.len());
        let mut prev_block = self.iv.clone();
        let mut pos = 0;

        while pos < data.len() {
            let key_stream = cipher.encrypt_block(&prev_block)?;
            let chunk_size = block_size.min(data.len() - pos);

            for i in 0..chunk_size {
                result.push(data[pos + i] ^ key_stream[i]);
            }

            prev_block = result[pos..pos + chunk_size].to_vec();
            pos += chunk_size;
        }

        Ok(result)
    }

    fn decrypt(&self, cipher: &dyn BlockCipher, data: &[u8]) -> Result<Vec<u8>, CipherError> {
        let block_size = cipher.block_size();

        if data.is_empty() {
            return Err(CipherError::DataTooShort);
        }

        if self.iv.len() != block_size {
            return Err(CipherError::InvalidIVLenght);
        }

        let mut result = Vec::with_capacity(data.len());
        let mut prev_block = self.iv.clone();
        let mut pos = 0;

        while pos < data.len() {
            let key_stream = cipher.encrypt_block(&prev_block)?;
            let chunk_size = block_size.min(data.len() - pos);

            for i in 0..chunk_size {
                result.push(data[pos + i] ^ key_stream[i]);
            }

            prev_block = vec![0u8; block_size];
            prev_block[..chunk_size].copy_from_slice(&data[pos..pos + chunk_size]);
            pos += chunk_size;
        }

        Ok(result)
    }

    fn is_block_mode(&self) -> bool {
        false
    }
}
