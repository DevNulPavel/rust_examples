#[cfg(test)]
use std::{
    collections::{
        HashMap
    }
};

pub trait TryParseParams: Sized {
    fn try_parse() -> Option<Self>;
}

#[cfg(test)]
pub trait TestableParams: Sized {
    fn get_keys() -> &'static [&'static str];
    fn test(values: &HashMap<String, String>);
}

