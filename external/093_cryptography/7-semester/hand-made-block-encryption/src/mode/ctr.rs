use crate::traits::{BlockCipher, Mode, CipherError};

#[derive(Debug, Clone)]
pub struct CTR {
    nonce: Vec<u8>
}

impl CTR {
    pub fn new(nonce: Vec<u8>) -> Self {
        Self { nonce }
    }

    fn generate_counter_block(&self, counter: &[u8], block_size: usize) -> Vec<u8> {
        let mut counter_block = vec![0u8; block_size];
        let part_size = block_size / 2;

        counter_block[..part_size].copy_from_slice(&self.nonce[..part_size]);
        counter_block[part_size..].copy_from_slice(counter);
        counter_block
    }

    fn increment_counter(counter: &mut [u8]) {
        for byte in counter.iter_mut().rev() {
            *byte = byte.wrapping_add(1);
            if *byte != 0 {
                break;
            }
        }
    }
}

impl Mode for CTR {
    fn encrypt(&self, cipher: &dyn BlockCipher, data: &[u8]) -> Result<Vec<u8>, CipherError> {
        let block_size = cipher.block_size();

        if data.is_empty() {
            return Err(CipherError::DataTooShort);
        }

        let mut result = Vec::with_capacity(data.len());
        let mut counter = vec![0u8; block_size / 2];
        let mut pos = 0;

        while pos < data.len() {
            let counter_block = self.generate_counter_block(&counter, block_size);
            let key_stream = cipher.encrypt_block(&counter_block)?;
            let chunk_size = block_size.min(data.len() - pos);

            for i in 0..chunk_size {
                result.push(data[pos + i] ^ key_stream[i]);
            }

            Self::increment_counter(&mut counter);
            pos += chunk_size;
        }

        Ok(result)
    }

    fn decrypt(&self, cipher: &dyn BlockCipher, data: &[u8]) -> Result<Vec<u8>, CipherError> {
        self.encrypt(cipher, data)
    }

    fn is_block_mode(&self) -> bool {
        false
    }
}
