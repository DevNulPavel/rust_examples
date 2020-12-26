use std::{
    env::{
        set_var
    },
    collections::{
        HashMap
    }
};
use rand::{
    distributions::{
        Alphanumeric
    },
    thread_rng, 
    Rng
};
use super::{
    traits::{
        EnvParams,
        EnvParamsTestable
    },
    *
};


macro_rules! test_type_before {
    ($map: ident, $type: ident) => {
        $map.extend(get_random_key_values::<$type>().into_iter());
    };
}

macro_rules! test_type_after {
    ($map: ident, $type: ident) => {
        $type::test(&$map);
    };
}

macro_rules! test_types {
    ($($type: ident),*) => {
        let mut test_values: HashMap<String, String> = HashMap::new();

        $( test_type_before!(test_values, $type);)*

        test_values
            .iter()
            .for_each(|(k, v)|{
                set_var(k, v);
            });

        $( test_type_after!(test_values, $type); )*
    };
}

fn rand_string() -> String{
    let rand_string: Vec<u8> = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .collect();
    
    std::str::from_utf8(&rand_string).unwrap().to_owned()
}

fn get_random_key_values<T: EnvParams + EnvParamsTestable>()-> HashMap<String, String>{
    let keys = T::get_available_keys();
    let res = keys
        .iter()
        .fold(HashMap::new(), |mut prev, key|{
            let key = key.to_string();
            prev.insert(key, rand_string());
            prev
        });
    res
}

#[test]
fn test_env_environment(){
    test_types! (
        GitEnvironment,
        AmazonEnvironment,
        AppCenterEnvironment,
        GooglePlayEnvironment,
        GoogleDriveEnvironment,
        IOSEnvironment,
        SSHEnvironment,
        TargetSlackEnvironment,
        ResultSlackEnvironment
    );
}