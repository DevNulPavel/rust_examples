use tracing::{
    instrument
};
use crate::{
    error::{
        AppError
    }
};

// TODO: Превратить в акторы

#[instrument(skip(password), fields(salt = ?salt.borrow()))]
pub async fn hash_password_with_salt<V>(password: V, salt: V) -> Result<String, AppError> 
where 
    V: std::borrow::Borrow<[u8]> + Send + 'static
{
    actix_web::rt::blocking::run(move || -> Result<String, argon2::Error> {
            let config = argon2::Config::default(); // TODO: Configure
            let res = argon2::hash_encoded(password.borrow(), salt.borrow(), &config)?;
            Ok(res)
        })
        .await
        .map_err(AppError::from)
}

#[instrument(skip(password))]
pub async fn verify_password<V>(password: V, password_hash: String) -> Result<bool, AppError> 
where 
    V: std::borrow::Borrow<[u8]> + Send + 'static
{
    actix_web::rt::blocking::run(move || -> Result<bool, argon2::Error> {
        let res = argon2::verify_encoded(&password_hash, password.borrow())?; // Верификация происходит достаточно быстро
        Ok(res)
    })
    .await
    .map_err(AppError::from)
}

#[cfg(test)]
mod tests{
    use super::*;

    #[actix_rt::test]
    async fn test_password_service(){

        let test_pass = b"asdasdasda".to_vec();
        let test_salt = b"test_salt_data".to_vec();
        let result_hash = hash_password_with_salt(test_pass.clone(), test_salt.clone())
            .await
            .expect("Hash calculate error");

        let pass_is_valid = verify_password(test_pass, result_hash)
            .await
            .expect("Verify failed");

        assert!(pass_is_valid, "Password verify failed");
    }
}