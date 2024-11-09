mod block_cipher;
mod mode;
mod padding;

pub use self::block_cipher::BlockCipher;
pub use self::mode::Mode;
pub use self::padding::Padding;

#[derive(Debug)]
pub enum CipherError {
    InvalidKeyLenght,
    InvalidIVLenght,
    InvalidBlockSize,
    DataTooShort,
    DataNotAligned,
    InvalidPadding,
    InvalidKeyFormat,
    InvalidMode,
}

#[derive(Debug)]
pub enum EncryptorType {
    Block(Box<dyn BlockCipher>, Box<dyn Mode>, Box<dyn Padding>),
    Stream(Box<dyn BlockCipher>, Box<dyn Mode>)
}

#[derive(Debug)]
pub struct Encryptor {
    encryptor_type: EncryptorType,
}

impl Encryptor {
    /// # Создание "Шифратора" для блочного режима шифрования
    pub fn new_block(
        cipher: Box<dyn BlockCipher>,
        mode: Box<dyn Mode>,
        padding: Box<dyn Padding>,
    ) -> Result<Self, CipherError> {
        if !mode.is_block_mode() {
            return Err(CipherError::InvalidMode);
        }

        Ok(Self {
            encryptor_type: EncryptorType::Block(cipher, mode, padding),
        })
    }

    /// # Создание "Шифратора" для поточного режима шифрования
    pub fn new_stream(
        cipher: Box<dyn BlockCipher>,
        mode: Box<dyn Mode>,
    ) -> Result<Self, CipherError> {
        if mode.is_block_mode() {
            return Err(CipherError::InvalidMode);
        }

        Ok(Self {
            encryptor_type: EncryptorType::Stream(cipher, mode),
        })
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, CipherError> {
        match &self.encryptor_type {
            EncryptorType::Block(cipher, mode, padding) => {
                let padded = padding.pad(data, cipher.block_size())?;
                mode.encrypt(&**cipher, &padded)
            },
            EncryptorType::Stream(cipher, mode) => {
                mode.encrypt(&**cipher, data)
            }
        }
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CipherError> {
        match &self.encryptor_type {
            EncryptorType::Block(cipher, mode, padding) => {
                let decrypted = mode.decrypt(&**cipher, data)?;
                padding.unpad(&decrypted)
            },
            EncryptorType::Stream(cipher, mode) => {
                mode.decrypt(&**cipher, data)
            }
        }
    }
}
