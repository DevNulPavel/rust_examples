use std::{
    fmt::{
        self,
        Display,
        Formatter
    },
    error::Error
};
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
pub enum CurrencyErrorType{
    RequestErr(reqwest::Error),
    InvalidResponse(&'static str),
    NoData(CurrencyType),
    NoSellInfo(CurrencyType),
    NoBuyInfo(CurrencyType),
    NoChangeInfo(CurrencyType)
}
error_from!(CurrencyErrorType, RequestErr, reqwest::Error);

///////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct CurrencyError{
    pub bank: &'static str,
    pub err_type: CurrencyErrorType
}

// TODO: Делать map_err для прописывания банка вместо ? на запросах
impl From<reqwest::Error> for CurrencyError{
    fn from(e: reqwest::Error) -> Self {
        CurrencyError{
            bank: "",
            err_type: CurrencyErrorType::RequestErr(e)
        }
    }
}

impl CurrencyError{
    pub fn new(bank: &'static str, err_type: CurrencyErrorType)->Self{
        CurrencyError{
            bank,
            err_type
        }
    }
}

impl Display for CurrencyError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Bank {} error: {:?}", self.bank, self.err_type)
    }
}

impl Error for CurrencyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
    fn description(&self) -> &str {
        "invalid first item to double"
    }
    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}