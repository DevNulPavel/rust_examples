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

pub fn file_is_not_empty_and_exists(path: impl AsRef<Path>, size_minimum: u64) -> bool {
    let file = match File::open(path){
        Ok(file) => file,
        Err(_) => return false
    };
    let meta = match file.metadata(){
        Ok(meta) => meta,
        Err(_) => return false
    };
    if meta.len() > size_minimum {
        true
    }else{
        false
    }
}

pub fn get_executable_full_path(name: &str) -> Option<String> {
    // TODO: Обработка ошибок
    let out = std::process::Command::new("which")
        .arg(name)
        .output()
        .expect("which command perform failed");
    if out.status.success() {
        let text = std::str::from_utf8(&out.stdout).expect("Utf8 parse failed").trim_end();
        if !text.is_empty(){
            Some(text.to_owned())
        }else{
            None
        }
    }else{
        None
    }
}