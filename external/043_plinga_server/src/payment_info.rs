use std::{
    collections::{
        HashMap
    }
};
use log::{
    // info,
    debug,
    // error
};
use serde::{
    Deserialize,
    Serialize
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
pub struct PaymentInfoRequest{
    signature: String,
    userid: String,
    #[serde(rename = "transactionId")]
    transaction_id: String,
    #[serde(flatten)]
    other: HashMap<String, String>
}
impl ValidateParams for PaymentInfoRequest {
    fn is_valid(&self, secret_key: &str) -> bool {
        // Проверяем валидность запроса
        let buffer = {
            let mut params_arr = vec![];
            // Обязательные параметры в алфавитном порядке
            params_arr.push(format!("transactionId={}", self.transaction_id));
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
            params_arr.push(secret_key.to_owned()); // TODO: Сow

            // Сцепляем параметры вместе
            let buffer = params_arr.join("&");

            buffer    
        };
        // Проверяем валидность запроса
        let result = format!("{:x}", md5::compute(buffer));
        debug!("Calculated signature: {}, received signature: {}", result, self.signature);
        self.signature.eq(&result)
    }
}

pub async fn get_payment_info(params: web::Form<PaymentInfoRequest>, shared_data: web::Data<SharedAppData>) -> Result<HttpResponse, actix_web::Error> {
    debug!("Payment info params: {:#?}", params);

    // Проверяем валидность запроса
    if !params.is_valid(&shared_data.secret_key) {
        return Err(HttpResponse::Forbidden().reason("Invalid hash").finish().into())
    }

    // TODO: Получить данные о деньгах для transaction_id

    #[derive(Debug, Serialize)]
    struct Response{
        success: bool,
        #[serde(rename = "currencyCode")]
        currency_code: String,
        #[serde(rename = "currencyAmount")]
        currency_amount: String, 
        #[serde(rename = "packageDescription")]
        package_description: String
    }
    let result = Response{
        success: true,
        currency_code: "EUR".to_string(),
        currency_amount: "1.24".to_string(),
        package_description: "Test description".to_string()
    };

    Ok(HttpResponse::Ok().json(result))
}