// Вспомогательная стандартная библиотека для работы с макросами
// use proc_macro::TokenStream as TokenStream1;

// Библиотека внешняя более удобная для работы с макросами
// use proc_macro2::TokenStream;

use quote::quote;
use syn::{parse_macro_input, parse_quote, Expr, ExprBinary, ItemFn};
