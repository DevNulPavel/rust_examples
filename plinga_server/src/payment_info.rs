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
            // Обязательные параметры
            params_arr.push(format!("userid={}", self.userid));
            params_arr.push(format!("transactionId={}", self.transaction_id));
            // Добавляем остальные итемы в параметрах
            let other_items_string = self.other
                .iter()
                .map(|(k, v)| {
                    format!("{}={}", k, v)
                });
            params_arr.extend(other_items_string);
            
            // Сцепляем параметры вместе
            let mut buffer = params_arr.join("&");

            // Добавляем секретный ключ
            buffer.push_str(secret_key);

            buffer    
        };
        // Проверяем валидность запроса
        let result = format!("{:x}", md5::compute(buffer));
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