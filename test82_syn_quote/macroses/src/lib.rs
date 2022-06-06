mod args;

// Вспомогательная стандартная библиотека для работы с макросами
// use proc_macro::TokenStream as TokenStream1;

// Библиотека внешняя более удобная для работы с макросами
// use proc_macro2::TokenStream;

use crate::args::Args;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{fold::Fold, parse_macro_input, ItemFn};

/// Аттрибут занимается тем, что печатает значения каждой переменной при переприсваивании
///
/// # Example
///
/// ```
/// #[macroses::trace_var(p, n)]
/// fn factorial(mut n: u64) -> u64 {
///     let mut p = 1;
///     while n > 1 {
///         p *= n;
///         n -= 1;
///     }
///     p
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn trace_var(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Сначала парсим входные токены в структуру данных, описывающую функцию
    let input = parse_macro_input!(input as ItemFn);

    // Затем парсим аргументы уже с помощью нашей реализации
    let mut args = parse_macro_input!(args as Args);

    // Use a syntax tree traversal to transform the function body.
    let output = args.fold_item_fn(input);

    // abort! { output,
    //     "I don't like this part!";
    //         note = "I see what you did there...";
    //         help = "I need only one part, you know?";
    // }

    // Hand the resulting function body back to the compiler.
    proc_macro::TokenStream::from(quote!(#output))
}
