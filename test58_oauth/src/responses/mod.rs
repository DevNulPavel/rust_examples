use std::{
    collections::{
        HashMap
    }
};
use serde::{
    Deserialize
};
use serde_json::{
    Value
};

////////////////////////////////////////////////////////////////////////

/// Специальный шаблонный тип, чтобы можно было парсить возвращаемые ошибки в ответах.
/// А после этого - конвертировать в результаты.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum DataOrErrorResponse<D, E>{
    Ok(D),
    Err(E)
}
impl<D, E> DataOrErrorResponse<D, E> {
    pub fn into_result(self) -> Result<D, E> {
        match self {
            DataOrErrorResponse::Ok(ok) => Ok(ok),
            DataOrErrorResponse::Err(err) => Err(err),
        }
    }
}

////////////////////////////////////////////////////////////////////////

/// Тип ошибки, в который мы можем парсить наши данные
#[derive(Deserialize, Debug)]
pub struct FacebookErrorValue{
    pub message: String,
    pub code: u32,
    pub error_subcode: u32,
    pub fbtrace_id: String,

    #[serde(rename = "type")]
    pub err_type: String
}
#[derive(Deserialize, Debug)]
pub struct FacebookErrorResponse{
    pub error: FacebookErrorValue,

    #[serde(flatten)]
    pub other: HashMap<String, Value>
}

////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
pub struct FacebookTokenResponse{
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,

    // #[serde(flatten)]
    // pub other: HashMap<String, Value>
}

////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
pub struct FacebookUserInfoResponse{
    pub id: String,
    pub name: String,

    #[serde(flatten)]
    pub other: HashMap<String, Value>
}

////////////////////////////////////////////////////////////////////////

/// Тип ошибки, в который мы можем парсить наши данные
// TODO: Формат данных
#[derive(Deserialize, Debug)]
pub struct GoogleErrorValue{
    pub message: String,
    pub code: u32,
    pub error_subcode: u32,
    pub fbtrace_id: String,

    #[serde(rename = "type")]
    pub err_type: String
}
#[derive(Deserialize, Debug)]
pub struct GoogleErrorResponse{
    pub error: GoogleErrorValue,

    #[serde(flatten)]
    pub other: HashMap<String, Value>
}

////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
pub struct GoogleTokenResponse{
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u32,

    // #[serde(flatten)]
    // pub other: HashMap<String, Value>
}

////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
pub struct GoogleUserInfoResponse{
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub picture: String,
    
    #[serde(flatten)]
    pub other: HashMap<String, Value>
}