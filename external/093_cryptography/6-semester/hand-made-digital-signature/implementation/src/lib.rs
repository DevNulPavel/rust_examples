use hand_made_el_gamal::ElGamal;
use hand_made_rsa::{TypeKey, RSA};
use pkcs7::EncryptionFunction;
use std::collections::HashMap;

/// "Обертка" структуры RSA для impl Trait"
pub struct Rsa {
    rsa: RSA,
}

impl Rsa {
    pub fn generate_keys() -> Self {
        Rsa {
            rsa: RSA::generate_keys(),
        }
    }
}

impl EncryptionFunction for Rsa {
    fn get_id(&self) -> String {
        "RSA".to_string()
    }

    fn get_public_key(&self) -> HashMap<String, String> {
        self.rsa.get_public_key()
    }

    fn encrypt(&self, message: &String) -> String {
        self.rsa.encrypt_msg(TypeKey::PrivateKey, &message)
    }
}

pub struct ElGamalWrapper {
    el_gamal: ElGamal,
}

impl ElGamalWrapper {
    pub fn generate_system() -> Self {
        ElGamalWrapper {
            el_gamal: ElGamal::generate_system(),
        }
    }
}

impl EncryptionFunction for ElGamalWrapper {
    fn get_id(&self) -> String {
        "ElGamal".to_string()
    }

    fn get_public_key(&self) -> HashMap<String, String> {
        self.el_gamal.get_public_key()
    }

    fn encrypt(&self, message: &String) -> String {
        let (y, delta) = self.el_gamal.sign_message(&message);
        y + "|" + delta.as_str()
    }
}
