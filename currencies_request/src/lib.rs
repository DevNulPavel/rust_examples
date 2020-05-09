#![warn(clippy::all)]

mod constants;
mod errors;
mod types;
mod alpha;
mod sber;
mod vtb;
mod central;

//use std::pin::Pin;
use reqwest::Client;
use futures::future::{
    //Future,
    FutureExt,
    //JoinAll
};
use crate::{
    alpha::get_currencies_from_alpha,
    central::get_currencies_from_central,
    sber::get_currencies_from_sber,
    vtb::get_currencies_from_vtb
};

pub use crate::{
    constants::PROXIES,
    errors::CurrencyError,
    types::{
        CurrencyResult,
        CurrencyChange,
    },
};

// type CurrenciesRequestFutureType = dyn Future<Output = Result<CurrencyResult<'static>, CurrencyError>> + std::marker::Send;
// type CurrenciesRequestReturnType = JoinAll<Pin<Box<CurrenciesRequestFutureType>>>;

// TODO: Избавиться от vec?
// использовать stream!
pub async fn get_all_currencies(client: &Client) -> Vec<Result<CurrencyResult<'static>, CurrencyError>> {
    // TODO: Посмотреть оборачивание в box + pin
    // TODO: Избавиться от vec?
    // https://users.rust-lang.org/t/expected-opaque-type-found-a-different-opaque-type-when-trying-futures-join-all/40596
    // https://users.rust-lang.org/t/expected-opaque-type-found-a-different-opaque-type-when-trying-futures-join-all/40596/5
    let futures_array = vec![
        get_currencies_from_central(&client, "Central").boxed(),
        get_currencies_from_alpha(&client, "Alpha").boxed(),
        get_currencies_from_vtb(&client, "VTB").boxed(),
        get_currencies_from_sber(&client, "Sber").boxed()
    ];

    let joined_futures = futures::future::join_all(futures_array);

    let result = joined_futures.await;
    result
}