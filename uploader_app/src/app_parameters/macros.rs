
macro_rules! params_data_type {
    (   
        $type_id:ident {
            $(
                Req {
                    $($val_rec:ident : $key_rec:literal : $desc_rec:literal),*
                }
            )?
            $(
                Opt{
                    $($val_opt:ident: $key_opt:literal : $desc_opt:literal),*
                }
            )?
            $(
                Mult{ 
                    $($val_mult:ident: $key_mult:literal : $desc_mult:literal),*
                }
            )?
            $(
                MultOpt{
                    $($val_mult_opt:ident: $key_mult_opt:literal : $desc_mult_opt:literal), *
                }
            )?
        }
    ) 
    => 
    {
        pub struct $type_id {
            $( $( pub $val_rec: String, )* )?
            $( $( pub $val_opt: Option<String>, )* )?
            $( $( pub $val_mult: Vec<String>, )* )?
            $( $( pub $val_mult_opt: Option<Vec<String>>, )* )?
        }
        impl crate::app_parameters::traits::AppParams for $type_id {
            fn get_args() -> Vec<clap::Arg<'static, 'static>>{
                vec![
                    $(
                        $(
                            clap::Arg::with_name($key_rec)
                                .long($key_rec)
                                .help($desc_rec)
                                .empty_values(false)
                                // .hidden(true)
                                .takes_value(true),
                        )*
                    )?
                    $(
                        $(
                            clap::Arg::with_name($key_opt)
                                .long($key_opt)
                                .help($desc_opt)
                                .empty_values(false)
                                // .hidden(true)
                                .takes_value(true),
                        )*
                    )?
                    $(
                        $(
                            clap::Arg::with_name($key_mult)
                                .long($key_mult)
                                .help($desc_mult)
                                .use_delimiter(true)
                                .value_delimiter(",")
                                .empty_values(false)
                                // .hidden(true)
                                .takes_value(true),
                        )*
                    )?
                    $(
                        $(
                            clap::Arg::with_name($key_mult_opt)
                                .long($key_mult_opt)
                                .help($desc_mult_opt)
                                .use_delimiter(true)
                                .value_delimiter(",")
                                .empty_values(false)
                                // .hidden(true)
                                .takes_value(true),
                        )*
                    )?
                ]
            }
            fn parse(values: &clap::ArgMatches) -> Option<Self> {
                Some($type_id {
                    $( $( $val_rec: values.value_of($key_rec)?.to_owned(), )* )?
                    $( $( $val_opt: values.value_of($key_opt).map(|v|{ v.to_owned() }), )* )?
                    $( $( $val_mult: 
                            values
                                .values_of($key_mult)?
                                .map(|v|{
                                    v.to_owned()
                                })
                                .collect::<Vec<String>>()
                    )* )?
                    $( $( $val_mult_opt: 
                            values
                                .values_of($key_mult_opt)
                                .map(|values|{
                                    let vector: Vec<String> = values
                                        .map(|v|{
                                            v.to_owned()
                                        })
                                        .collect();
                                    vector
                                }), 
                    )* )?
                })
            }
        }
    };
}