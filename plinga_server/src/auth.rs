use log::{
    // info,
    debug,
    // error
};
use serde::{
    Deserialize
};
use actix_web::{
    web::{
        self
    },
    HttpResponse
};
use crate::{
    shared_data::{
        SharedAppData
    }, 
    validate_params::{
        ValidateParams
    }
};

#[derive(Deserialize, Debug)]
pub struct AuthRequest{
    userid: String,
    sessionkey: String,
    sessionid: String,
    lang: String,
    platform: String,
    #[serde(rename = "platformSPIL")]
    platform_spil: String
}
impl ValidateParams for AuthRequest {
    fn is_valid(&self, secret_key: &str) -> bool {
        let buffer = {
            let mut buffer = String::new();
            buffer.push_str(&self.userid);
            buffer.push_str(&self.sessionkey);
            buffer.push_str(secret_key);
            buffer    
        };
        // Хэш от данных и полученный идентификатор должны совпасть
        let result = format!("{:x}", md5::compute(buffer));
        self.sessionid.eq(&result)
    }
}

pub async fn auth(params: web::Query<AuthRequest>, 
             shared_data: web::Data<SharedAppData>) -> Result<HttpResponse, actix_web::Error> {
    debug!("Auth request with params: {:#?}", params.0);
    
    if params.is_valid(&shared_data.secret_key) {
        Ok(HttpResponse::Ok().finish())
    }else{
        Err(HttpResponse::Forbidden().reason("Invalid hash").finish().into())
    }
}