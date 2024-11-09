use hand_made_math::expanded_euclidean_algorithm;
use hash_function_wrapper::{HashFunc, HashFunction};
use num_bigint::{BigInt, RandBigInt, ToBigInt};
use num_primes::Generator;
use num_traits::{Num, Zero};
use pkcs7::EncryptionFunction;
use std::collections::HashMap;

const BIT_SIZE: usize = 128;

/// # Структура ключей системы Фиата-Шамира
#[derive(Default, Debug)]
pub struct FiatShamir {
    public_key: PublicKey,
    private_key: PrivateKey,
    hash_function: HashFunc,
}

/// # Структура публичного ключа системы Фиата-Шамира
#[derive(Default, Debug)]
pub struct PublicKey {
    n: BigInt,
    b: Vec<BigInt>,
}

impl PublicKey {
    /// # Создание ключа из хэш-маппы
    /// Используется для восстановления ключа из JSON файла
    pub fn from_hashmap(hash_map: HashMap<String, String>) -> PublicKey {
        let b = hash_map["b"]
            .split("|")
            .map(|b| BigInt::from_str_radix(b, 10).unwrap())
            .collect::<Vec<BigInt>>();

        PublicKey {
            n: BigInt::from_str_radix(hash_map["n"].as_str(), 10).unwrap(),
            b,
        }
    }
}

/// # Структура приватного ключа системы Фиата-Шамира
#[derive(Default, Debug)]
struct PrivateKey {
    p: BigInt,
    q: BigInt,
    a: Vec<BigInt>,
}

impl EncryptionFunction for FiatShamir {
    fn get_id(&self) -> String {
        "FiatShamir".to_string()
    }

    fn encrypt(&self, message: &String) -> String {
        self.sign_message(&message)
    }

    fn get_public_key(&self) -> HashMap<String, String> {
        self.get_public_key()
    }
}

impl FiatShamir {
    /// # Генерация значений системы
    /// В ходе генерации будут получены числа для
    /// формирования публичного и приватного ключей
    pub fn generate_system(hash_function: &dyn HashFunction) -> Self {
        let p = BigInt::from_bytes_be(
            num_bigint::Sign::Plus,
            &Generator::new_prime(BIT_SIZE).to_bytes_be(),
        );
        let q = BigInt::from_bytes_be(
            num_bigint::Sign::Plus,
            &Generator::new_prime(BIT_SIZE).to_bytes_be(),
        );
        let n = &p * &q;

        let mut a = Vec::new();
        let mut b = Vec::new();

        for _ in 0..hash_function.get_hash_size() {
            a.push(rand::thread_rng().gen_bigint(10));
        }

        for ai in a.iter() {
            let (_, mut inverse_a, _) = expanded_euclidean_algorithm(ai.clone(), n.clone());
            if inverse_a < BigInt::zero() {
                inverse_a += &n;
            }

            b.push(inverse_a.modpow(&BigInt::from(2), &n));
        }

        FiatShamir {
            public_key: PublicKey { b, n },
            private_key: PrivateKey { a, p, q },
            hash_function: hash_function.get_hash_enum(),
        }
    }

    /// # Создание подписи
    /// Создает подпись для данного сообщения с
    /// использованием данной хэш-функции
    pub fn sign_message(&self, message: &str) -> String {
        let r = rand::thread_rng()
            .gen_bigint_range(&BigInt::from(1), &BigInt::from(&self.public_key.n - 1));
        let u = r.modpow(&BigInt::from(2), &self.public_key.n);

        let hash = self
            .hash_function
            .hash(&(message.to_string() + u.to_string().as_str()));

        dbg!(&self.hash_function);

        let mut s: Vec<u32> = vec![0u32; self.hash_function.get_hash_size()];
        let temp_s = BigInt::from_str_radix(&hash, 16)
            .unwrap()
            .to_str_radix(2)
            .chars()
            .map(|char| char.to_digit(10).unwrap())
            .collect::<Vec<u32>>();
        s[(self.hash_function.get_hash_size() - temp_s.len())..self.hash_function.get_hash_size()]
            .copy_from_slice(&temp_s);

        let mut t = r;

        for i in 0..self.private_key.a.len() {
            t *= &self.private_key.a[i].modpow(&s[i].to_bigint().unwrap(), &self.public_key.n);
        }
        t = &t % &self.public_key.n;

        hash + "|" + t.to_str_radix(16).as_str()
    }

    /// # Проверка цифровой подписи
    /// Проверяет подпись путем сравнивания данной с
    /// вычисленной для данного сообщения
    pub fn check_signature(&self, signature: String, message: String) -> bool {
        let (hash, t) = signature.split_once("|").unwrap();
        let t = BigInt::from_str_radix(&t, 16).unwrap();
        let n = self.public_key.n.clone();

        let mut s: Vec<u32> = vec![0u32; self.hash_function.get_hash_size()];
        let temp_s = BigInt::from_str_radix(&hash, 16)
            .unwrap()
            .to_str_radix(2)
            .chars()
            .map(|char| char.to_digit(10).unwrap())
            .collect::<Vec<u32>>();
        s[(self.hash_function.get_hash_size() - temp_s.len())..self.hash_function.get_hash_size()]
            .copy_from_slice(&temp_s);

        let mut w = &t * &t;

        for i in 0..self.public_key.b.len() {
            w *= &self.public_key.b[i].modpow(&s[i].to_bigint().unwrap(), &n);
        }
        w = &w % &n;

        let s_for_check = self.hash_function.hash(&(message + w.to_string().as_str()));

        dbg!(&hash);
        dbg!(&s_for_check);

        hash.to_string() == s_for_check
    }

    /// # Получение публичного ключа
    pub fn get_public_key(&self) -> HashMap<String, String> {
        let mut hashmap = HashMap::new();
        let public_key = self
            .public_key
            .b
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
            .join("|");

        hashmap.insert("b".to_string(), public_key);
        hashmap.insert("n".to_string(), self.public_key.n.to_string());

        hashmap
    }

    /// # Установка публичного ключа
    pub fn set_public_key(&mut self, public_key: PublicKey) {
        self.public_key = public_key;
    }

    pub fn set_hash_func(&mut self, hash_func: HashFunc) {
        self.hash_function = hash_func;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fiat_shamir_digital_signature() {
        let message = "Hello World!";
        let fiat_shamir = FiatShamir::generate_system(&HashFunc::Sha256);
        let signature = fiat_shamir.sign_message(message);

        assert_eq!(
            fiat_shamir.check_signature(signature, message.to_string()),
            true
        );
    }
}
