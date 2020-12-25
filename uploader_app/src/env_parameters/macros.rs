// https://doc.rust-lang.org/reference/macros-by-example.html
// https://doc.rust-lang.org/reference/procedural-macros.html#attribute-macros
// https://doc.rust-lang.org/book/ch19-06-macros.html
// https://doc.rust-lang.org/edition-guide/rust-2018/macros/custom-derive.html
// https://doc.rust-lang.org/book/ch19-06-macros.html#how-to-write-a-custom-derive-macro
macro_rules! data_type {
    ($type: ident, $($val: ident: $key: literal),*) => {
        pub struct $type {
            $($val: String,)*
        }

        impl crate::env_parameters::traits::TryParseParams for $type {
            fn try_parse() -> Option<Self> {
                Some(Self{
                    $($val: var($key).ok()?),*
                })
            }
        }

        #[cfg(test)]
        impl crate::env_parameters::traits::TestableParams for $type {
            fn test(values: &std::collections::HashMap<String, String>){
                let val = Self::try_parse()
                    .expect(&format!("Failed to parse: {}", stringify!($type)));
                $( assert_eq!(val.$val.eq(&values[$key]), true); )*
            }
            fn get_keys() -> &'static [&'static str] {
                let keys = &[
                    $($key,)*
                ];
                keys
            }
        }
    };
}
