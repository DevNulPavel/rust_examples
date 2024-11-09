use std::fmt::Debug;
use super::CipherError;

pub trait Padding: Send + Sync + Debug {
    fn pad(&self, data: &[u8], block_size: usize) -> Result<Vec<u8>, CipherError>;
    fn unpad(&self, data: &[u8]) -> Result<Vec<u8>, CipherError>;
}
