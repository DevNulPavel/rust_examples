use hand_made_math::expanded_euclidean_algorithm;
use num_bigint::BigInt;
use num_primes::Generator;
use num_traits::Num;
use std::collections::HashMap;

/// # Размерность чисел p и q в битах
const PQ_BIT_SIZE: usize = 256;

/// Генерация простого числа заданной размерности
pub fn gen_prime(bits: usize) -> BigInt {
    Generator::new_prime(bits)
        .to_str_radix(10)
        .parse::<BigInt>()
        .unwrap()
}

/// Создание открытого ключа из своих данных
pub fn create_public_key(e: String, n: String) -> PublicKey {
    PublicKey {
        e: BigInt::from_str_radix(&e, 10).unwrap(),
        n: BigInt::from_str_radix(&n, 10).unwrap(),
    }
}
/// Генерация случайных ключей
pub fn generate_keys() -> (PrivateKey, PublicKey) {
    let e = gen_prime(16);
    let p = gen_prime(PQ_BIT_SIZE);
    let q = gen_prime(PQ_BIT_SIZE);

    let public_key = PublicKey {
        e: e.clone(),
        n: &p * &q,
    };

    let (_, mut d, _) = expanded_euclidean_algorithm(e, (&p - 1) * (&q - 1));

    if &d < &BigInt::from(0) {
        d += (&p - 1) * (&q - 1);
    };

    let private_key = PrivateKey { p, q, d };

    (private_key, public_key)
}

/// Открытый ключ
#[derive(Clone)]
pub struct PublicKey {
    e: BigInt,
    n: BigInt,
}

impl PublicKey {
    /// Получить части открытого ключа
    pub fn get_raw_parts(&self) -> (BigInt, BigInt) {
        (self.e.clone(), self.n.clone())
    }

    /// Сформировать открытый ключ с помощью HashMap <String, String> содержащая в себе ключ "e" и "n"
    pub fn create_key_from_hashmap(data: HashMap<String, String>) -> PublicKey {
        PublicKey {
            e: BigInt::from_str_radix(data["e"].as_str(), 10).unwrap(),
            n: BigInt::from_str_radix(data["n"].as_str(), 10).unwrap(),
        }
    }
}

/// # Закрытый ключ
#[derive(Clone)]
pub struct PrivateKey {
    p: BigInt,
    q: BigInt,
    d: BigInt,
}

impl PrivateKey {
    /// Получить части закрытого ключа
    pub fn get_raw_parts(&self) -> (BigInt, BigInt, BigInt) {
        (self.p.clone(), self.q.clone(), self.d.clone())
    }

    /// Собрать закрытый ключ из частей
    pub fn from_raw_parts(p: &str, q: &str, d: &str) -> PrivateKey {
        PrivateKey {
            p: BigInt::from_str_radix(p, 10).unwrap(),
            q: BigInt::from_str_radix(q, 10).unwrap(),
            d: BigInt::from_str_radix(d, 10).unwrap(),
        }
    }

    /// Сформировать приватный ключ с помощью HashMap <String, String>
    pub fn create_key_from_hashmap(data: HashMap<String, String>) -> PrivateKey {
        PrivateKey {
            p: BigInt::from_str_radix(data["p"].as_str(), 10).unwrap(),
            q: BigInt::from_str_radix(data["q"].as_str(), 10).unwrap(),
            d: BigInt::from_str_radix(data["d"].as_str(), 10).unwrap(),
        }
    }
}
