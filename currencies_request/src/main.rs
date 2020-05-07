#![warn(clippy::all)]

extern crate chrono;

mod errors;
mod alpha;
mod types;

use tokio::{
    runtime::{
        Builder,
        Runtime
    }
};
use crate::{
    errors::CurrencyError,
    types::CurrencyResult,
    alpha::get_currencies_from_alpha,
};

async fn async_main(){
    let result: Result<CurrencyResult, CurrencyError> = get_currencies_from_alpha().await;

    println!("{:?}", result);
}

fn main() {
    // Создаем однопоточный рантайм, здесь нет нужды в многопоточном
    let mut runtime: Runtime = Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap();
    
    runtime.block_on(async_main());
}