// https://doc.rust-lang.org/reference/macros-by-example.html
// https://doc.rust-lang.org/reference/procedural-macros.html#attribute-macros
// https://doc.rust-lang.org/book/ch19-06-macros.html
// https://doc.rust-lang.org/edition-guide/rust-2018/macros/custom-derive.html
// https://doc.rust-lang.org/book/ch19-06-macros.html#how-to-write-a-custom-derive-macro
macro_rules! env_params_type {
    (
        $type: ident { 
            $(
                Req{
                    $($val_req:ident : $key_req:literal),*
                }
            )?
            $(
                Opt{
                    $($val_opt:ident : $key_opt:literal),*
                }
            )?
        }
    ) => {
        #[derive(Debug)]
        pub struct $type {
            $( $(pub $val_req: String,)* )?
            $( $(pub $val_opt: Option<String>,)* )?
        }
        impl crate::env_parameters::traits::EnvParams for $type {
            fn try_parse() -> Option<Self> {
                Some(Self{
                    $( $( $val_req: var($key_req).ok().map(|val| val.trim_matches('"').to_string() )?, )* )?
                    $( $( $val_opt: var($key_opt).ok().map(|val| val.trim_matches('"').to_string() ), )* )?
                })
            }
            fn get_available_keys() -> &'static [&'static str] {
                let keys = &[
                    $( $($key_req,)* )?
                    $( $($key_opt,)* )?
                ];
                keys
            }
        }
        #[cfg(test)]
        impl crate::env_parameters::traits::EnvParamsTestable for $type {
            fn test(values: &std::collections::HashMap<String, String>){
                let val: Self = crate::env_parameters::traits::EnvParams::try_parse()
                    .expect(&format!("Failed to parse: {}", stringify!($type)));

                $( $( assert_eq!(val.$val_req.eq(&values[$key_req]), true); )* )?
                $( $(
                        {
                            let test_1 = val.$val_opt;
                            let test_2 = values.get($key_opt);
                            match (test_1, test_2){
                                (Some(ref v1), Some(ref v2)) => {
                                    assert_eq!((&v1).eq(v2), true);
                                },
                                (Some(_), None) | (None, Some(_)) => {
                                    panic!("Test failed");
                                },
                                _ => {
                                }
                            }
                        }
                    )* 
                )?
            }
            
        }
    };
}
