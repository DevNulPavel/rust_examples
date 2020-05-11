// use std::{
    // time::Duration,
    // collections::HashMap,
// };
// use chrono::prelude::*;
use reqwest::{
    Client,
    // ClientBuilder,
};
// use serde::{
    // Deserialize, 
    // Serialize
// };
use crate::{
    errors::{
        CurrencyError,
        CurrencyErrorType::*,
    },
    types::{
        CurrencyResult,
        // CurrencyValue,
        // CurrencyChange,
        CurrencyType::{
            // self,
            // EUR,
            USD
        },
    }
};
// use derive_new::new;

pub async fn get_currencies_from_central(_client: &Client, bank_name: &'static str) -> Result<CurrencyResult, CurrencyError> {
    // Создаем клиента для запроса
    // let client: Client = ClientBuilder::new()
    //     .connect_timeout(Duration::from_secs(3))
    //     .timeout(Duration::from_secs(3))
    //     .build()?;

    // Получаем json
    // "https://alfabank.ru/ext-json/0.2/exchange/cash?offset=0&limit=1&mode=rest"
    // let json: HashMap<String, Vec<AlphaCurrency>> = client
    //     .get("https://alfabank.ru/ext-json/0.2/exchange/cash")
    //     .query(&[("offset", "0"), ("mode", "rest")])
    //     .send()
    //     .await?
    //     .json()
    //     .await?;

    //println!("{:?}", json);
    
    Err(CurrencyError::new(bank_name, NoData(USD)))
}