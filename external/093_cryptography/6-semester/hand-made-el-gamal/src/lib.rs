use std::collections::HashMap;
use num_bigint::{BigInt, RandBigInt, Sign};
use rand::{thread_rng};
use num_traits::Num;
use hand_made_math::expanded_euclidean_algorithm;
use num_primes::Generator;

#[derive(Default)]
pub struct ElGamal {
    p: BigInt,
    g: BigInt,
    public_key: PublicKey,
    private_key: PrivateKey,
}

impl ElGamal {
    pub fn generate_system() -> Self {
        let p = gen_prime(512);
        let mut alpha = thread_rng().gen_bigint_range(&BigInt::from(1), &BigInt::from(&p-1));

        while alpha.modpow(&BigInt::from(2), &p) == BigInt::from(1) || alpha.modpow(&(&p/2), &p) == BigInt::from(1) || alpha.modpow(&BigInt::from(2), &p) == BigInt::from(&p-1){
            alpha = thread_rng().gen_bigint_range(&BigInt::from(1), &BigInt::from(&p-1));
        }
        let a = thread_rng().gen_bigint_range(&BigInt::from(1), &BigInt::from(&p-2));
        let beta = alpha.modpow(&a, &p);

        ElGamal {
            p: p.clone(),
            g: alpha.clone(),
            public_key: PublicKey {
                beta,
                alpha,
                p
            },
            private_key: PrivateKey {
                a
            }
        }
    }
    pub fn sign_message(&self, message: &str) -> (String, String) {
        let message_to_number = BigInt::from_bytes_be(Sign::Plus, message.as_bytes());

        let mut r = thread_rng().gen_bigint_range(&BigInt::from(1), &BigInt::from(&self.p - 2));

        while expanded_euclidean_algorithm(r.clone(), &self.p - 1).0 != BigInt::from(1) {
            r = thread_rng().gen_bigint_range(&BigInt::from(1), &BigInt::from(&self.p - 2));
        };

        let y = self.g.modpow(&r, &self.p);
        let (_, mut u, _) = expanded_euclidean_algorithm(r.clone(), &self.p - 1);
        if u < BigInt::from(0) {
            u += &self.p - 1;
        }
        let mut delta: BigInt = ((message_to_number - &self.private_key.a * &y) * &u) % (&self.p - 1);
        if delta < BigInt::from(0) {
            delta += &self.p - 1;
        }
        (y.to_str_radix(16), delta.to_str_radix(16))
    }

    pub fn check_signature(&self, signature: (String, String), message: &str) -> bool {
        let message_to_number = BigInt::from_bytes_be(Sign::Plus, message.as_bytes());
        let y = BigInt::from_str_radix(signature.0.as_str(), 16).unwrap();
        let delta = BigInt::from_str_radix(signature.1.as_str(), 16).unwrap();

        let left = (self.public_key.beta.modpow(&y, &self.public_key.p) *
            y.modpow(&delta, &self.public_key.p)) % &self.public_key.p;
        let right = self.public_key.alpha.modpow(&message_to_number, &self.public_key.p);
        right == left

    }

    pub fn set_public_key(&mut self, public_key: PublicKey) {
        self.public_key = public_key;
    }

    pub fn get_public_key(&self) -> HashMap<String, String> {
        let mut hash_map = HashMap::new();
        hash_map.insert("alpha".to_string(), self.public_key.alpha.to_string());
        hash_map.insert("beta".to_string(), self.public_key.beta.to_string());
        hash_map.insert("p".to_string(), self.public_key.p.to_string());
        hash_map
    }
}

#[derive(Default, Debug)]
pub struct PublicKey {
    p: BigInt,
    alpha: BigInt,
    beta: BigInt,
}

impl PublicKey {
    pub fn from_hashmap(data: HashMap<String, String>) -> PublicKey {
        PublicKey {
            p: BigInt::from_str_radix(data["p"].as_str(),10).unwrap(),
            alpha: BigInt::from_str_radix(data["alpha"].as_str(),10).unwrap(),
            beta: BigInt::from_str_radix(data["beta"].as_str(),10).unwrap(),
        }
    }
}


#[derive(Default)]
struct PrivateKey {
    a: BigInt,
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_el_gamal() {
        let message = "H";
        let el_gamal = ElGamal::generate_system();
        let key = el_gamal.get_public_key();
        let mut el_gamal_2 = ElGamal::default();
        el_gamal_2.set_public_key(PublicKey::from_hashmap(key));
        let digital_sign = el_gamal.sign_message(&message);
        let check = el_gamal_2.check_signature(digital_sign, message);

        assert_eq!(check, true);
    }
}

fn gen_prime(bits: usize) -> BigInt {
    Generator::new_prime(bits).to_str_radix(10).parse::<BigInt>().unwrap()
}
