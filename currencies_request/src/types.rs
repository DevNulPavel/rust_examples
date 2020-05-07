use derive_new::new;
use chrono::prelude::*;

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

#[derive(new, Debug)]
pub struct CurrencyValue{
    pub cur_type: CurrencyType,
    pub buy: f32,
    pub sell: f32,
    pub buy_change: CurrencyChange,
    pub sell_change: CurrencyChange,
}

#[derive(new, Debug)]
pub struct CurrencyResult{
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