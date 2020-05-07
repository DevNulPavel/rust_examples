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

#[derive(Debug, Copy, Clone)]
pub enum CurrencyType{
    EUR,
    USD
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

#[derive(new, Debug)]
pub struct CurrencyValue{
    pub cur_type: CurrencyType,
    pub buy: f32,
    pub sell: f32,
    pub buy_change: CurrencyChange,
    pub sell_change: CurrencyChange,
}

#[derive(new, Debug)]
pub struct CurrencyResult<'a>{
    pub bank_name: &'a str,
    pub usd: CurrencyValue,
    pub eur: CurrencyValue,
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