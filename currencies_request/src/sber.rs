use std::{
    //time::Duration,
    collections::HashMap,
};
use chrono::prelude::*;
use reqwest::{
    Client,
    //ClientBuilder,
};
use serde::{
    Deserialize, 
    Serialize
};
use crate::{
    errors::{
        CurrencyError,
        //CurrencyErrorType,
        CurrencyErrorType::*
    },
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
//use derive_new::new;

#[derive(Deserialize, Debug)]
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
    date: i64,
}

fn change_by_values(cur: f32, prev: f32) -> CurrencyChange{
    if cur < prev {
        CurrencyChange::Decrease
    }else if cur > prev {
        CurrencyChange::Increase
    }else{
        CurrencyChange::NoChange
    }
}

impl CurrencyValue {
    fn from_sber(cur_type: CurrencyType, info: &SberCurrency) -> Self {
        let buy_change = change_by_values(info.buy, info.buy_prev);
        let sell_change = change_by_values(info.sell, info.sell_prev);
        let res: CurrencyValue = CurrencyValue::new(cur_type, info.sell, info.buy, buy_change, sell_change);

        res
    }
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
        .ok_or(CurrencyError::new(bank_name, InvalidResponse("No cards info in sber")))?;

    let usd_info = json
        .get("840")
        .ok_or(CurrencyError::new(bank_name, NoData(USD)))?
        .get("0")
        .ok_or(CurrencyError::new(bank_name, NoData(USD)))?;

    let eur_info = json
        .get("978")
        .ok_or(CurrencyError::new(bank_name, NoData(USD)))?
        .get("0")
        .ok_or(CurrencyError::new(bank_name, NoData(USD)))?;
    
    //println!("{:?}", usd_info);
    //println!("{:?}", eur_info);
        
    let usd: CurrencyValue = CurrencyValue::from_sber(USD, usd_info);
    let eur: CurrencyValue = CurrencyValue::from_sber(EUR, eur_info);

    let native_time = NaiveDateTime::from_timestamp(usd_info.date / 1000, 0);
    let time = DateTime::<Utc>::from_utc(native_time, Utc);

    let result: CurrencyResult = CurrencyResult::new(bank_name, usd, eur, Some(time));

    Ok(result)
}