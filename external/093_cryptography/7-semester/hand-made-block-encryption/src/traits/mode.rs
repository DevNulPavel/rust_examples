use std::fmt::Debug;
use super::{BlockCipher, CipherError};

pub trait Mode: Send + Sync + Debug {
    fn encrypt(&self, cipher: &dyn BlockCipher, data: &[u8]) -> Result<Vec<u8>, CipherError>;
    fn decrypt(&self, cipher: &dyn BlockCipher, data: &[u8]) -> Result<Vec<u8>, CipherError>;
    fn is_block_mode(&self) -> bool;
}
