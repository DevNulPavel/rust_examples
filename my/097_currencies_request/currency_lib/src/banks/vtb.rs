use std::{
    collections::HashMap,
};
// use chrono::prelude::*;
use reqwest::{
    Client,
};
use serde::{
    Deserialize, 
};
use crate::{
    errors::{
        CurrencyError,
        CurrencyErrorType
    },
    types::{
        CurrencyResult,
        CurrencyValue,
        //CurrencyChange,
        CurrencyType::{
            self,
            EUR,
            USD
        },
    }
};

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

impl CurrencyValue {
    fn from_vtb(cur_type: CurrencyType, info: &VtbMoneyRate) -> Self {
        CurrencyValue{
            cur_type: cur_type,
            buy: info.sell,
            sell: info.buy,
            buy_change: info.sell_raised.into(),
            sell_change: info.buy_raised.into()
        }
    }
}

fn find_rate_for_currencies<'a>(rated: &'a Vec<HashMap<String, Vec<VtbMoneyRate>>>, from: &str, to: &str) -> Option<&'a VtbMoneyRate>{
    let found: Option<&VtbMoneyRate> = rated
        .iter()
        .filter_map(|hash_map|{
            hash_map.get("MoneyRates")
        })
        .flat_map(|rates_vec|{
            rates_vec.iter()
        })
        .find(|rate|{
            rate.from_currency.code.eq(from) && 
                rate.to_currency.code.eq(to)
        });
    found
}

pub async fn get_currencies_from_vtb(client: &Client, bank_name: &'static str) -> Result<CurrencyResult, CurrencyError> {
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

    // println!("\n{:?}\n", json);

    let usd_to_rub: &VtbMoneyRate = find_rate_for_currencies(&json.rated, "USD", "RUR")
        .ok_or_else(||{
            CurrencyError::new(bank_name, CurrencyErrorType::NoData(USD))
        })?;
    let eur_to_rub: &VtbMoneyRate = find_rate_for_currencies(&json.rated, "EUR", "RUR")
        .ok_or_else(||{
            CurrencyError::new(bank_name, CurrencyErrorType::NoData(EUR))
        })?;

    // println!("{:?}", usd_to_rub.date);
    // println!("{:?}", eur_to_rub);

    use chrono::offset::TimeZone;

    let time = match chrono::NaiveDateTime::parse_from_str(usd_to_rub.date.as_str(), "%Y-%m-%dT%H:%M:%S"){
        Ok(time) => {
            match chrono::offset::Local.from_local_datetime(&time).single(){
                Some(local_time) => {
                    let local_time: chrono::DateTime<chrono::Local> = local_time;
                    Some(local_time.with_timezone(&chrono::Utc))
                },
                None => {
                    None
                }
            }
        },
        Err(_) => {
            None
        }
    };
    
    let res = CurrencyResult{
        bank_name: bank_name.to_string(),
        usd: CurrencyValue::from_vtb(USD, &usd_to_rub),
        eur: CurrencyValue::from_vtb(EUR, &eur_to_rub),
        update_time: time
    };

    Ok(res)
}