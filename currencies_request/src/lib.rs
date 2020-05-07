#![warn(clippy::all)]

mod errors;
mod types;
mod alpha;
mod sber;
mod central;

pub use crate::{
    errors::CurrencyError,
    types::{
        CurrencyResult,
        CurrencyChange,
    },
    alpha::get_currencies_from_alpha,
    central::get_currencies_from_central,
    sber::get_currencies_from_sber
};