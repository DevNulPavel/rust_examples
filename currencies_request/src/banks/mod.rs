mod alpha;
mod sber;
mod central;
mod vtb;

pub use crate::{
    banks::{
        alpha::get_currencies_from_alpha,
        central::get_currencies_from_central,
        sber::get_currencies_from_sber,
        vtb::get_currencies_from_vtb
    }
};