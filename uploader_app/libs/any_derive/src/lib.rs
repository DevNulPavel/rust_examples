/*use proc_macro::{
    TokenStream
};
use syn::{
    spanned::{
        Spanned
    },
    parse_macro_input, 
    parse_quote, 
    Data, 
    DeriveInput, 
    Fields, 
    GenericParam, 
    Generics, 
    Index
};
use quote::{
    quote, 
    quote_spanned
};


// https://github.com/dtolnay/syn/blob/b341361011acf8fe93082e795864ad797f83e608/examples/heapsize/heapsize_derive/src/lib.rs
fn any_is_some_fields(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let values = fields
                        .named
                        .iter()
                        .map(|f| {
                            let name = &f.ident;
                            quote_spanned!{
                                f.span() => || #name.is_some() 
                            }
                        });
                    quote! {
                        false #(#values)*
                    }
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

// Add a bound `T: HeapSize` to every type parameter T.
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(AnyIsSome));
        }
    }
    generics
}

#[proc_macro_derive(AnyIsSome)]
pub fn any_option_is_some(input: TokenStream) -> TokenStream {
    // Парсим входные токены в синтаксическое дерево
    let input = parse_macro_input!(input as DeriveInput);

    // Имя нашего класса
    let name = input.ident;

    // Add a bound `T: HeapSize` to every type parameter T.
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Содержимое нашего кода
    let data = any_is_some_fields(&input.data);

    // Создаем вывод
    let expanded = quote! {
        impl #impl_generics AnyIsSome for #name #ty_generics #where_clause  {
            fn any_is_some(&self) -> bool {
                #data
            }
        }
    };

    // Возвращаем выходные токены компилятору
    TokenStream::from(expanded)
}


trait AnyIsSome {
    /// Total number of bytes of heap memory owned by `self`.
    fn any_is_some(&self) -> bool;
}*/
