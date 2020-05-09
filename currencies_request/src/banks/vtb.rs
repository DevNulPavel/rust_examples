use std::{
    //time::Duration,
    collections::HashMap,
};
// use chrono::prelude::*;
use reqwest::{
    Client,
    //ClientBuilder,
};
use serde::{
    Deserialize, 
    // Serialize
};
use crate::{
    errors::{
        CurrencyError,
        //CurrencyErrorType,
        CurrencyErrorType::*
    },
    types::{
        CurrencyResult,
        // CurrencyValue,
        // CurrencyChange,
        // CurrencyType::{
        //     self,
        //     EUR,
        //     USD
        // },
    }
};
//use derive_new::new;

#[derive(Deserialize, Debug)]
struct VtbCurrencyName{
    #[serde(rename(deserialize = "Code"))]
    code: String,
}

#[derive(Deserialize, Debug)]
struct VtbMoneyRate{
    #[serde(rename(deserialize = "FromCurrency"))]
    from_currency: VtbCurrencyName,

    #[serde(rename(deserialize = "ToCurrency"))]
    to_currency: VtbCurrencyName,

    #[serde(rename(deserialize = "StartDate"))]
    date: String,

    #[serde(rename(deserialize = "BankSellAt"))]
    sell: f32,

    #[serde(rename(deserialize = "BankBuyAt"))]
    buy: f32,

    #[serde(rename(deserialize = "IsBankSellAtRaised"))]
    sell_raised: bool,
    
    #[serde(rename(deserialize = "IsBankBuyAtRaised"))]
    buy_raised: bool
}

#[derive(Deserialize, Debug)]
struct VtbCurrencyResponse{
    // https://serde.rs/field-attrs.html

    #[serde(rename(deserialize = "GroupedRates"))]
    rated: Vec<HashMap<String, Vec<VtbMoneyRate>>>,

    #[serde(rename(deserialize = "DateFrom"))]
    date: String,
}

// impl CurrencyValue {
//     fn from_vtb(cur_type: CurrencyType, info: &VtbCurrency) -> Self {
//         res
//     }
// }

pub async fn get_currencies_from_vtb(client: &Client, bank_name: &'static str) -> Result<CurrencyResult<'static>, CurrencyError> {
    // Получаем json
    let url = "https://www.vtb.ru/api/currency-exchange/table-info\
                ?contextItemId=%7BC5471052-2291-4AFD-9C2D-1DBC40A4769D%7D\
                &conversionPlace=1\
                &conversionType=1\
                &renderingId=ede2e4d0-eb6b-4730-857b-06fd4975c06b\
                &renderingParams=LegalStatus__%7BF2A32685-E909-44E8-A954-1E206D92FFF8%7D\
                    ;IsFromRuble__1\
                    ;CardMaxPeriodDays__5\
                    ;CardRecordsOnPage__5\
                    ;ConditionsUrl__%2Fpersonal%2Fplatezhi-i-perevody%2Fobmen-valjuty%2Fspezkassy%2F\
                    ;Multiply100JPYand10SEK__1";
    let json: VtbCurrencyResponse = client
        .get(url)
        .send()
        .await?
        .json()
        .await?;

    println!("{:?}", json);

    Err(CurrencyError::new(bank_name, InvalidResponse("Test")))
}