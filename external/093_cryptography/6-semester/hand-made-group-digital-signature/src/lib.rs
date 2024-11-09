use rand::thread_rng;
use hand_made_sha::sha256;
use num::{BigInt, One, Zero};
use num_bigint::RandBigInt;
use num_primes::Generator;
use primal::Primes;

const DEFAULT_KEY_LENGHT: usize = 64;

#[derive(Debug)]
pub struct Sign {
    U: BigInt,
    E: BigInt,
    S: BigInt,
}

#[derive(Debug)]
pub struct KeyComponents {
    p: BigInt,
    q: BigInt,
    alpha: BigInt,
}

impl KeyComponents {
    pub fn new() -> KeyComponents {
        let p: BigInt = BigInt::from_bytes_be(num::bigint::Sign::Plus, &Generator::new_prime(DEFAULT_KEY_LENGHT).to_bytes_be());

        let q: BigInt = Primes::all()
            .skip_while(|prime| BigInt::from(*prime) < BigInt::from(547))
            .take_while(|prime| BigInt::from(*prime) < (&p - 1) / 2)
            .find(|prime| (&p - 1) % BigInt::from(*prime) == BigInt::from(0))
            .unwrap()
            .into();

        let mut alpha = rand::thread_rng().gen_bigint_range(&BigInt::one(), &BigInt::from(&p - 1));
        while alpha.modpow(&BigInt::from(2), &p) == BigInt::one() ||
            alpha.modpow(&((&p - 1) / 2), &p) == BigInt::one() ||
            alpha.modpow(&BigInt::from(2), &p) == BigInt::from(&p - 1) {
            alpha = thread_rng().gen_bigint_range(&BigInt::one(), &BigInt::from(&p - 1));
        };
        alpha.modpow(&((&p - 1) / &q), &p);

        KeyComponents {
            p,
            q,
            alpha,
        }
    }

    pub fn get_p(&self) -> &BigInt {
        &self.p
    }

    pub fn get_q(&self) -> &BigInt {
        &self.q
    }

    pub fn get_alpha(&self) -> &BigInt {
        &self.alpha
    }
}

#[derive(Debug)]
pub struct PersonalKeys {
    public_key: BigInt,
    private_key: BigInt,
}

impl PersonalKeys {
    pub fn new(kc: &KeyComponents) -> PersonalKeys {
        let private_key: BigInt = BigInt::from_bytes_be(num::bigint::Sign::Plus, &Generator::new_prime(DEFAULT_KEY_LENGHT).to_bytes_be());
        let public_key: BigInt = kc.get_alpha().modpow(&private_key, kc.get_p());

        PersonalKeys {
            public_key,
            private_key,
        }
    }

    pub fn get_public_key(&self) -> &BigInt {
        &self.public_key
    }

    pub fn get_private_key(&self) -> &BigInt {
        &self.private_key
    }
}

#[derive(Debug)]
pub struct IngroupKeys {
    p_1: BigInt,
    p_2: BigInt,
    n: BigInt,
    e: BigInt,
    d: BigInt,
}

impl IngroupKeys {
    pub fn new() -> IngroupKeys {
        let p_1: BigInt = BigInt::from_bytes_be(num::bigint::Sign::Plus, &Generator::new_prime(DEFAULT_KEY_LENGHT).to_bytes_be());
        let p_2: BigInt = BigInt::from_bytes_be(num::bigint::Sign::Plus, &Generator::new_prime(DEFAULT_KEY_LENGHT).to_bytes_be());
        let n: BigInt = &p_1 * &p_2;
        let e: BigInt = BigInt::from(65537);
        let d: BigInt = e.modinv(&((&p_1 - 1) * (&p_2 - 1))).unwrap();

        IngroupKeys {
            p_1,
            p_2,
            n,
            e,
            d,
        }
    }

    pub fn get_p_1(&self) -> &BigInt {
        &self.p_1
    }

    pub fn get_p_2(&self) -> &BigInt {
        &self.p_2
    }

    pub fn get_n(&self) -> &BigInt {
        &self.n
    }

    pub fn get_e(&self) -> &BigInt {
        &self.e
    }

    pub fn get_d(&self) -> &BigInt {
        &self.d
    }
}

pub fn check_sign(
    E: &BigInt,
    probably_E: &BigInt,
) -> bool {
    true
}