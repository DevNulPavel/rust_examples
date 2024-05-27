/*

use std::{
    path::{
        Path,
        PathBuf
    },
    ffi::{
        OsStr
    },
    process::{
        Command,
        Child,
        Output,
        //ExitStatus,
        //ExitCode,
        //Termination
    },
    env::{
        args
    },
    io::{
        self,
        Read
    }
};

enum AppExitStatus{
    Ok,
    NoCompiler,
    CompilerError(i32)
}

impl Termination for AppExitStatus{
    fn report(self) -> i32{
        match self {
            AppExitStatus::Ok => {
                0
            },
            AppExitStatus::NoCompiler => {
                -1
            },
            AppExitStatus::CompilerError(e) => {
                e
            }
        }
    }
}


use serde::{    
    Deserialize
};


#[derive(Deserialize, Debug)]
struct Configuration {
    env_variables: std::collections::hash_map::HashMap<String, String>
}

impl Configuration{
    fn env_to_arg_iter<'a>(&'a self) -> impl Iterator<Item=(&'a OsStr, &'a OsStr)>{
        let cccache_params_iter = {
            self.env_variables
                .iter()
                .map(|(key,val)|{
                    (std::ffi::OsStr::new(key), std::ffi::OsStr::new(val))
                })
        };
        cccache_params_iter
    }
}

fn read_configuration_file(filename: &str) -> Configuration {
    let current_executable_path = std::env::current_exe()
        .expect("Current executable get path failed");

    let executable_folder = current_executable_path
        .parent()
        .expect("Current executable get directory failed");

    let config_path = executable_folder.join(filename);

    let text = match std::fs::read_to_string(&config_path){
        Ok(text) => text,
        Err(e) => {
            let path_str = config_path
                .to_str()
                .expect("Failed to get path str");
            panic!("Failed to read config file {}, {}", path_str, e)
        }
    };

    let config = match serde_json::from_str(&text) {
        Ok(conf) => conf,
        Err(e) => {
            let path_str = config_path
                .to_str()
                .expect("Failed to get path str");
            panic!("Failed to parse config file {}: {}", path_str, e)
        }
    };

    config
}*/
