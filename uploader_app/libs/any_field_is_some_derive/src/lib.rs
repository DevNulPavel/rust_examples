use syn::{
    parse_macro_input, 
    Data, 
    DeriveInput, 
    Fields
};
use quote::{
    quote
};

// https://github.com/dtolnay/syn/tree/master/examples/heapsize

fn any_is_some_impl(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => {
                    let values = fields
                        .named
                        .iter()
                        .map(|field| {
                            let name = &field.ident;
                            quote!{
                                || self.#name.is_some()
                            }
                        });
                    let res = quote! {
                        false #(#values)*
                    };

                    res.into()
                }
                Fields::Unnamed(_) | Fields::Unit => {
                    unimplemented!()
                }
            }
        }
        Data::Enum(_) | Data::Union(_) => {
            unimplemented!()
        }
    }
}

#[proc_macro_derive(AnyFieldIsSome)]
pub fn any_field_is_some_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Парсим входные токены в синтаксическое дерево
    let input = parse_macro_input!(input as DeriveInput);

    // Имя нашего класса
    let name = input.ident;

    // Содержимое нашего кода
    let data: proc_macro2::TokenStream = any_is_some_impl(&input.data);

    // Создаем вывод
    let expanded = quote! {
        impl AnyFieldIsSome for #name  {
            fn any_field_is_some(&self) -> bool {
                #data
            }
        }
    };

    // Возвращаем выходные токены компилятору
    proc_macro::TokenStream::from(expanded)
}