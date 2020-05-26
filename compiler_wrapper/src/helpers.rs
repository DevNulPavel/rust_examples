use std::{
    path::{
        Path
    },
    fs::{
        File
    },
    env::{
        self
    }
};

pub fn is_env_var_enabled(var_name: &str) -> bool {
    match env::var(var_name){
        Ok(val) => {
            val.eq("1") || val.eq("true")
        },
        Err(_) => {
            false
        }
    }
}

pub fn file_is_not_empty_and_exists(path: impl AsRef<Path>) -> bool {
    let file = match File::open(path){
        Ok(file) => file,
        Err(_) => return false
    };
    let meta = match file.metadata(){
        Ok(meta) => meta,
        Err(_) => return false
    };
    if meta.len() > 3 {
        true
    }else{
        false
    }
}