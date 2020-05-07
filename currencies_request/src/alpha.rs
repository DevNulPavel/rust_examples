use std::{
    time::Duration,
    collections::HashMap
};
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
        CurrencyValue,
        CurrencyType::{
            self,
            EUR,
            USD
        }
    }
};
use derive_new::new;


#[derive(Serialize, Deserialize, Debug)]
struct AlphaCurrency{
    #[serde(rename(serialize = "type", deserialize = "type"))] // https://serde.rs/field-attrs.html
    type_val: String,
    date: String,
    value: f32,
    order: String
}


#[derive(new)]
struct BuyAndSellInfo<'a>{
    buy: &'a AlphaCurrency,
    sell: &'a AlphaCurrency
}

fn get_buy_and_sell(info: &Vec<AlphaCurrency>, cur_type: CurrencyType) -> Result<BuyAndSellInfo, CurrencyError>{
    let buy: &AlphaCurrency = info
        .iter()
        .find(|val|{
            val.type_val.eq("buy")
        })
        .ok_or(CurrencyError::NoBuyInfo(cur_type))?;

    let sell: &AlphaCurrency = info
        .iter()
        .find(|val|{
            val.type_val.eq("sell")
        })
        .ok_or(CurrencyError::NoSellInfo(cur_type))?;

    Ok(BuyAndSellInfo::new(buy, sell))
}

pub async fn get_currencies_from_alpha() -> Result<CurrencyValue, CurrencyError> {
    let client: Client = ClientBuilder::new()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(3))
        .build()?;

    let json: HashMap<String, Vec<AlphaCurrency>> = client
        .get("https://alfabank.ru/ext-json/0.2/exchange/cash?offset=0&limit=1&mode=rest")
        .send()
        .await?
        .json()
        .await?;

    let usd: &Vec<AlphaCurrency> = json
        .get("usd")
        .ok_or(CurrencyError::NoData(USD))?;

    let eur: &Vec<AlphaCurrency> = json
        .get("eur")
        .ok_or(CurrencyError::NoData(EUR))?;

    let usd_result = get_buy_and_sell(usd, USD)?;
    let eur_result = get_buy_and_sell(eur, EUR)?;

    // TODO: Изменение значений и дата

    Ok(CurrencyValue::new(usd_result.buy.value, 
                          usd_result.sell.value, 
                          eur_result.buy.value, 
                          eur_result.sell.value))
}