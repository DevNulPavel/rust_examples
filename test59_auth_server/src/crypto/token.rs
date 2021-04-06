use uuid::{
    Uuid
};
use chrono::{
    Utc,
    Duration
};
use serde::{
    Deserialize,
    Serialize
};
use tracing::{
    instrument
};
use crate::{
    error::{
        AppError
    }
};

// TODO: Превратить в акторы

/////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims{
    pub uuid: Uuid,
    pub exp: i64 // Имя должно быть обязательно такое, иначе библиотека jsonwebtoken не находит время жизни токена
}
impl TokenClaims {
    pub fn is_not_expired(&self) -> bool {
        let current_ts = Utc::now().timestamp();
        current_ts < self.exp
    }

    #[cfg(test)]
    pub fn is_valid_for_uuid(&self, uuid: &Uuid) -> bool {
        self.uuid.eq(uuid) && self.is_not_expired()
    }

    /*pub fn get_uuid(&self) -> &Uuid {
        &self.uuid
    }*/
}

/////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize)]
pub struct TokenGenerateResult{
    pub token: String,
    pub token_type: String,
    pub expires_in: i64
}

/////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct TokenService{
    secret_key: String
}
impl TokenService {
    pub fn new(secret_key: String) -> TokenService {
        TokenService {
            secret_key
        }
    }

    #[instrument]
    pub async fn generate_jwt_token(&self, uuid: Uuid) -> Result<TokenGenerateResult, AppError>{
        let encoding_key = jsonwebtoken::EncodingKey::from_secret(self.secret_key.as_bytes());
        actix_web::rt::blocking::run(move || -> Result<TokenGenerateResult, jsonwebtoken::errors::Error> {
                let header = jsonwebtoken::Header::default();
                // Время, когда истекает токен
                let expire_time = Utc::now() + Duration::days(1); // Expires in 1 day
                // Метаинформация о токене
                let claims = TokenClaims{
                    uuid,
                    exp: expire_time.timestamp()
                };
                // Генерируем токен
                let res = jsonwebtoken::encode(&header, &claims, &encoding_key)?;
                Ok(TokenGenerateResult{
                    token: res,
                    token_type: "Bearer".to_string(),
                    expires_in: expire_time.timestamp()
                })
            })
            .await
            .map_err(AppError::from)
    }

    #[instrument]
    pub async fn decode_jwt_token(&self, token: String) -> Result<TokenClaims, AppError>{
        let key = self.secret_key.clone();
        actix_web::rt::blocking::run(move || -> Result<TokenClaims, jsonwebtoken::errors::Error> {
                let decoding_key = jsonwebtoken::DecodingKey::from_secret(key.as_bytes());
                let validation = jsonwebtoken::Validation::default();
                let res = jsonwebtoken::decode(&token, &decoding_key, &validation)?;
                Ok(res.claims)
            })
            .await
            .map_err(AppError::from)
    }
}

/////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests{
    use super::*;

    #[actix_rt::test]
    async fn test_token_service(){
        let service = TokenService::new("this_is_the_secret_key".to_string());

        let user_id = Uuid::new_v4();

        let result_token = service
            .generate_jwt_token(user_id.clone())
            .await
            .expect("Token generate failed");

        let token_claims = service
            .decode_jwt_token(result_token.token)
            .await
            .expect("Decode token failed");

        println!("Claims: {:#?}", token_claims);

        assert!(token_claims.is_valid_for_uuid(&user_id), "Token is not valid");
    }
}