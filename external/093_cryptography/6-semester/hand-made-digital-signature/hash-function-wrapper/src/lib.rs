use hand_made_sha::{sha256, sha512};
use hand_made_streebog::{streebog_256, streebog_512};

/// # Возможные хэш-функции
#[derive(Default, Debug, Clone)]
pub enum HashFunc {
    #[default]
    Sha256,
    Sha512,
    Streebog256,
    Streebog512,
}

/// # Трейт для использования произвольных хэш-функций в PKCS 7
pub trait HashFunction {
    /// # Возвращает идентификатор хэш-функции
    fn get_id(&self) -> String;

    /// # Возвращает размер хэша
    fn get_hash_size(&self) -> usize;

    /// # Возвращает значение перечисления
    fn get_hash_enum(&self) -> HashFunc;

    /// # Возвращает хэш
    fn hash(&self, message: &String) -> String;
}

impl HashFunction for HashFunc {
    fn get_id(&self) -> String {
        match self {
            HashFunc::Sha256 => "SHA256".to_string(),
            HashFunc::Sha512 => "SHA512".to_string(),
            HashFunc::Streebog256 => "STREEBOG256".to_string(),
            HashFunc::Streebog512 => "STREEBOG512".to_string(),
        }
    }

    fn get_hash_size(&self) -> usize {
        match self {
            HashFunc::Sha256 => 256,
            HashFunc::Sha512 => 512,
            HashFunc::Streebog256 => 256,
            HashFunc::Streebog512 => 512,
        }
    }

    fn hash(&self, message: &String) -> String {
        match self {
            HashFunc::Sha256 => sha256(message.as_bytes())
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .collect(),

            HashFunc::Sha512 => sha512(message.as_bytes())
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .collect(),

            HashFunc::Streebog256 => streebog_256(message.as_bytes())
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .collect(),

            HashFunc::Streebog512 => streebog_512(message.as_bytes())
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .collect(),
        }
    }

    fn get_hash_enum(&self) -> HashFunc {
        self.clone()
    }
}
