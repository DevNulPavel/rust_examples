use derive_new::new;

#[derive(Debug, Copy, Clone)]
pub enum CurrencyType{
    EUR,
    USD
}

#[derive(new, Debug)]
pub struct CurrencyValue{
    pub usd_buy: f32,
    pub usd_sell: f32,
    pub eur_buy: f32,
    pub eur_sell: f32
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