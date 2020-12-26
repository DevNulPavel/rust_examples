#[cfg(test)]
use std::{
    collections::{
        HashMap
    }
};

pub trait EnvParams: Sized {
    fn try_parse() -> Option<Self>;
    fn get_available_keys() -> &'static [&'static str];
}

#[cfg(test)]
pub trait EnvParamsTestable: Sized {
    fn test(values: &HashMap<String, String>);
}

