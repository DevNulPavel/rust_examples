use std::{
    collections::{
        HashMap
    }
};
// use log::{
    // info,
    // debug,
    // error
// };
use serde::{
    Deserialize,
    // Serialize
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

#[derive(Deserialize)]
pub struct ProcessPayRequest{
    userid: String,
    game: String,
    ts: String,
    coins: String,
    price: String,
    currency: String,
    tid: String,
    oid: String,
    signature: String,
    #[serde(flatten)]
    other: HashMap<String, String>
}
impl ValidateParams for ProcessPayRequest {
    fn is_valid(&self, secret_key: &str) -> bool {
        // Проверяем валидность запроса
        let buffer = {
            let mut params_arr = vec![];
            // Обязательные параметры в алфавитном порядке
            params_arr.push(format!("coins={}", self.coins));
            params_arr.push(format!("currency={}", self.currency));
            params_arr.push(format!("game={}", self.game));
            params_arr.push(format!("oid={}", self.oid));
            params_arr.push(format!("price={}", self.price));
            params_arr.push(format!("tid={}", self.tid));
            params_arr.push(format!("ts={}", self.ts));
            params_arr.push(format!("userid={}", self.userid));
            // Добавляем остальные итемы в параметрах
            // TODO: Вынести в функцию
            let mut other_params = self
                .other
                .iter()
                .collect::<Vec<_>>();
            other_params
                .sort_by(|a, b| {
                    a.0.to_lowercase().cmp(&b.0.to_lowercase())
                });
            let other_items_iter = other_params
                .iter()
                .map(|(k, v)| {
                    format!("{}={}", k, v)
                });
            params_arr.extend(other_items_iter);
            
            // Добавляем секретный ключ
            params_arr.push(secret_key.to_owned()); // TODO: Cow

            // Сцепляем параметры вместе
            let buffer = params_arr.join("&");

            buffer    
        };
        // Проверяем валидность запроса
        let result = format!("{:x}", md5::compute(buffer));
        self.signature.eq(&result)
    }
}

pub async fn process_pay(params: web::Form<ProcessPayRequest>, shared_data: web::Data<SharedAppData>) -> Result<HttpResponse, actix_web::Error> {
    // Проверяем валидность запроса
    if !params.is_valid(&shared_data.secret_key) {
        return Err(HttpResponse::Forbidden().reason("Invalid hash").finish().into())
    }

    Ok(HttpResponse::Ok().finish())
}
