use std::{
    sync::{
        Arc
    }
};
use crate::{
    error::{
        AppError
    }
};

#[derive(Debug, Clone)]
pub struct CryptoService{
    // pub salt: Arc<String>
}

impl CryptoService {
    pub fn new() -> CryptoService {
        CryptoService{
            // salt: Arc::new(key)
        }
    }

    // TODO: 

    async fn hash_password_with_salt<V>(&self, password: V, salt: V) -> Result<String, AppError> 
    where 
        V: std::borrow::Borrow<[u8]> + Send + 'static
    {
        let fut = actix_web::rt::blocking::run(move || -> Result<String, argon2::Error> {
            let config = argon2::Config::default(); // TODO: Configure
            let res = argon2::hash_encoded(password.borrow(), salt.borrow(), &config)?;
            // TODO: argon2::verify_encoded(encoded, pwd)
            Ok(res)
        });
        fut
            .await
            .map_err(|err|{
                match err {
                    actix_web::rt::blocking::BlockingError::Canceled => {
                        AppError::PasswordHashSpawnError
                    },
                    actix_web::rt::blocking::BlockingError::Error(e) => {
                        AppError::from(e)
                    }
                }
            })
    }
}

// TODO: Unit test