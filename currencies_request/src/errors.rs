use crate::{
    types::CurrencyType
};

macro_rules! error_from {
    ($err_struct: ty, $enum_val: ident, $source_type: ty) => {
        impl From<$source_type> for $err_struct{
            fn from(e: $source_type) -> Self {
                Self::$enum_val(e)
            }
        }
    };
    ($err_struct: ty, $enum_val: ident, $source_type: ty, $convert_expr: ident) => {
        impl From<$source_type> for $err_struct{
            fn from(e: $source_type) -> Self {
                Self::$enum_val(e.$convert_expr())
            }
        }
    };
}

#[derive(Debug)]
pub enum CurrencyError{
    RequestErr(reqwest::Error),
    InvalidResponse(&'static str),
    NoData(CurrencyType),
    NoSellInfo(CurrencyType),
    NoBuyInfo(CurrencyType),
    NoChangeInfo(CurrencyType)
}
error_from!(CurrencyError, RequestErr, reqwest::Error);

