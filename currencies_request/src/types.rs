use std::fmt::{
    self,
    Display,
    Formatter
};
use derive_new::new;
use chrono::prelude::*;
//use crate::errors::CurrencyError;

// pub trait BankRequestFuture: futures::Future {
//     type Output = Result<CurrencyResult<'static>, CurrencyError>;
// }

// #[derive(Debug, Copy, Clone)]
// pub enum BankType{
//     Central,
//     Alpha
// }

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CurrencyType{
    EUR,
    USD
}

impl Into<&'static str> for &CurrencyType {
    fn into(self) -> &'static str {
        match self {
            CurrencyType::USD =>{
                "USD"
            },
            CurrencyType::EUR =>{
                "EUR"
            }
        }
    }
}

impl From<CurrencyType> for &'static str {
    fn from(val: CurrencyType) -> &'static str {
        match val {
            CurrencyType::USD =>{
                "USD"
            },
            CurrencyType::EUR =>{
                "EUR"
            }
        }
    }
}

impl Display for CurrencyType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let s: &'static str = self.into();
        write!(f, "{}", s)
    }
}

impl std::convert::TryFrom<&str> for CurrencyType{
    // TODO: ??
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error>{
        match value {
            "EUR" => Ok(Self::EUR),
            "USD" => Ok(Self::USD),
            _ => Err(())
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum CurrencyChange{
    Increase,
    Decrease,
    NoChange
}

impl Display for CurrencyChange {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Increase =>{
                write!(f, "↑")
            },
            Self::Decrease =>{
                write!(f, "↓")
            },
            Self::NoChange => {
                write!(f, "=")
            }
        }
    }
}

#[derive(new, Debug, Clone)]
pub struct CurrencyValue{
    pub cur_type: CurrencyType,
    pub buy: f32,
    pub sell: f32,
    pub buy_change: CurrencyChange,
    pub sell_change: CurrencyChange,
}

#[derive(new, Debug, Clone)]
pub struct CurrencyResult {
    pub bank_name: String,
    pub usd: CurrencyValue,
    pub eur: CurrencyValue,
    pub update_time: Option<DateTime<Utc>>
}

#[derive(new, Debug, Clone, PartialEq)]
pub struct CurrencyMinimum {
    pub bank_name: String,
    pub value: f32,
    pub cur_type: CurrencyType,
    pub update_time: Option<DateTime<Utc>>
}

/*impl CurrencyValue{
    fn new(usd_buy: f32, usd_sell: f32, eur_buy: f32, eur_sell: f32){
        CurrencyValue{
            usd_buy,
            usd_sell,
            eur_buy,
            eur_sell
        }
    }
}*/