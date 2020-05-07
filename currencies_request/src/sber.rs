use std::{
    time::Duration,
    collections::HashMap,
};
use chrono::prelude::*;
use reqwest::{
    Client,
    ClientBuilder,
};
use serde::{
    Deserialize, 
    Serialize
};
use crate::{
    errors::CurrencyError,
    types::{
        CurrencyResult,
        CurrencyValue,
        CurrencyChange,
        CurrencyType::{
            self,
            EUR,
            USD
        },
    }
};
use derive_new::new;

#[derive(Serialize, Deserialize, Debug)]
struct SberCurrency{
    // https://serde.rs/field-attrs.html
    #[serde(rename(deserialize = "isoCur"))]
    type_val: String,

    #[serde(rename(deserialize = "buyValue"))]
    buy: f32,
    #[serde(rename(deserialize = "sellValue"))]
    sell: f32,

    #[serde(rename(deserialize = "buyValuePrev"))]
    buy_prev: f32,
    #[serde(rename(deserialize = "sellValuePrev"))]
    sell_prev: f32,

    #[serde(rename(deserialize = "activeFrom"))]
    date: u64,
}

pub async fn get_currencies_from_sber(client: &Client, bank_name: &'static str) -> Result<CurrencyResult<'static>, CurrencyError> {
    // Получаем json
    let url = "https://www.sberbank.ru/portalserver/proxy/\
               ?pipe=shortCachePipe\
               &url=http%3A%2F%2Flocalhost%2Frates-web%2FrateService%2Frate%2Fcurrent%3FregionId%3D77%26rateCategory%3Dcards%26currencyCode%3D978%26currencyCode%3D840";
    // let url = "https://www.sberbank.ru/portalserver/proxy/\
    //            ?pipe=shortCachePipe\
    //            &url=http%3A%2F%2Flocalhost%2Frates-web%2FrateService%2Frate%2Fcurrent%3FregionId%3D77%26rateCategory%3Dcards%26currencyCode%3D840"; 
    let json: HashMap<String, HashMap<String, HashMap<String, SberCurrency>>> = client
        .get(url)
        .send()
        .await?
        .json()
        .await?;
    
    let json = json
        .get("cards")
        .ok_or(CurrencyError::InvalidResponse("No cards info in sber"))?;

    let usd_info = json
        .get("840")
        .ok_or(CurrencyError::NoData(USD))?
        .get("0")
        .ok_or(CurrencyError::NoData(USD))?;

    let eur_info = json
        .get("978")
        .ok_or(CurrencyError::NoData(USD))?
        .get("0")
        .ok_or(CurrencyError::NoData(USD))?;
    
    println!("{:?}", usd_info);
    println!("{:?}", eur_info);
        
    Err(CurrencyError::NoData(USD))
}