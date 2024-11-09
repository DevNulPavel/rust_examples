use crate::keys::{PrivateKey, PublicKey};
use num_bigint::{BigInt, Sign};
use num_traits::Num;
use std::collections::HashMap;

/// Модуль для работы с ключами
pub mod keys;

/// Тип используемого ключа для шифрования
pub enum TypeKey {
    PublicKey,
    PrivateKey,
}

/// Система шифрования
pub struct RSA {
    private_key: PrivateKey,
    public_key: PublicKey,
}

impl RSA {
    /// Установить свой публичный ключ
    pub fn set_public_key(&mut self, public_key: PublicKey) {
        self.public_key = public_key
    }
    /// Создать систему на основе закрытого и открытого ключа
    pub fn create(private_key: PrivateKey, public_key: PublicKey) -> RSA {
        RSA {
            private_key,
            public_key,
        }
    }
    /// Шифрование сообщения, type_key - выбор ключа на котором будет выполнено шифрование
    pub fn encrypt_msg(&self, type_key: TypeKey, message: &String) -> String {
        let (e, n) = self.public_key.get_raw_parts();
        let (_, _, d) = self.private_key.get_raw_parts();

        let mut block_bytes = Vec::new();

        let binding = message.as_bytes().to_vec();
        for el in binding.chunks(64) {
            block_bytes.push(el);
        }

        match type_key {
            TypeKey::PrivateKey => {
                let encrypted_block_bytes = block_bytes
                    .iter()
                    .map(|el| {
                        BigInt::from_bytes_be(Sign::Plus, el)
                            .modpow(&d, &n)
                            .to_str_radix(16)
                    })
                    .collect::<Vec<String>>()
                    .join("|");
                encrypted_block_bytes
            }
            TypeKey::PublicKey => {
                let encrypted_block_bytes = block_bytes
                    .iter()
                    .map(|el| {
                        BigInt::from_bytes_be(Sign::Plus, el)
                            .modpow(&e, &n)
                            .to_str_radix(16)
                    })
                    .collect::<Vec<String>>()
                    .join("|");
                encrypted_block_bytes
            }
        }
    }

    /// Расшифрование сообщения, type_key - выбор ключа на котором будет выполнено шифрование
    pub fn decrypt_msg(&self, type_key: TypeKey, encrypted_message: &String) -> String {
        let (e, n) = self.public_key.get_raw_parts();
        let (p, q, d) = self.private_key.get_raw_parts();
        match type_key {
            TypeKey::PrivateKey => {
                let messages = encrypted_message.split("|").collect::<Vec<&str>>();
                let un_hex = messages
                    .iter()
                    .map(|el| BigInt::from_str_radix(el, 16).unwrap())
                    .collect::<Vec<BigInt>>();
                let decode_block_bytes = un_hex
                    .iter()
                    .map(|el| el.modpow(&d, &(&q * &p)).to_bytes_be().1)
                    .collect::<Vec<Vec<u8>>>();
                let mut output = String::new();

                for el in decode_block_bytes {
                    output += String::from_utf8(el).unwrap().as_str();
                }

                output
            }
            TypeKey::PublicKey => {
                let messages = encrypted_message.split("|").collect::<Vec<&str>>();
                let un_hex = messages
                    .iter()
                    .map(|el| BigInt::from_str_radix(el, 16).unwrap())
                    .collect::<Vec<BigInt>>();
                let decode_block_bytes = un_hex
                    .iter()
                    .map(|el| el.modpow(&e, &n).to_bytes_be().1)
                    .collect::<Vec<Vec<u8>>>();
                let mut output = String::new();

                for el in decode_block_bytes {
                    output += String::from_utf8(el).unwrap().as_str();
                }

                output
            }
        }
    }

    /// Создать систему на основе генерации случайных ключей
    pub fn generate_keys() -> Self {
        let keys = keys::generate_keys();
        RSA {
            public_key: keys.1,
            private_key: keys.0,
        }
    }

    /// Получить публичный ключ в виде HashMap<String, String>, Keys - поля открытого ключа
    pub fn get_public_key(&self) -> HashMap<String, String> {
        let mut public_key = HashMap::new();
        let (e, n) = self.public_key.get_raw_parts();
        public_key.insert("e".to_string(), e.to_string());
        public_key.insert("n".to_string(), n.to_string());
        public_key
    }

    /// Получить приватный ключ в виде HashMap<String, String>, Keys - поля приватного ключа
    pub fn get_private_key(&self) -> HashMap<String, String> {
        let mut private_key = HashMap::new();
        let (p, q, d) = self.private_key.get_raw_parts();
        private_key.insert("p".to_string(), p.to_string());
        private_key.insert("q".to_string(), q.to_string());
        private_key.insert("d".to_string(), d.to_string());
        private_key
    }
}

#[cfg(test)]
mod tests {
    use super::RSA;
    use crate::TypeKey;
    #[test]
    fn test_rsa() {
        let rsa = RSA::generate_keys();
        let message = "1fd59ce875833cc493ca3657c1075018a7de0c0cdb1ccab43e645b977c6f6f0a742ac1c467d6f4b92f9350098e322fa29755a2cd8c8488ed2fbcab5aa431eb1cf72f6674d571ba871a44e06963ef1f8bfff5f05b5c74c861cf2588cf976c7943eac89d7927d6af3585ab11e6f36484aa28e5538c36d4d7883b5ec83c9096a6d7".to_string();
        let encrypted_msg = rsa.encrypt_msg(TypeKey::PrivateKey, &message);
        dbg!(&encrypted_msg);
        let decrypted_msg = rsa.decrypt_msg(TypeKey::PublicKey, &encrypted_msg);
        assert_eq!(decrypted_msg, message)
    }
}
