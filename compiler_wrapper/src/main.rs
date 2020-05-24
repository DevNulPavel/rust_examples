#![warn(clippy::all)]
// #![feature(lang_items)]
// #![feature(start)]
// #![no_std]

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
        //ExitCode,
        //ExitStatus,
        //Termination
    },
    env::{
        args
    },
    io::{
        self
    }
};

/*enum AppExitStatus{
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
}*/

fn file_is_not_empty_and_exists(path: &Path) -> bool {
    match std::fs::read_to_string(path){
        Ok(data) => {
            if data.len() > 0{
                return true;
            }else{
                return false;
            }
        },
        Err(_) => {
            return false;
        }
    }
}

/// Итератор по параметрам CCcache
fn build_cccache_params_iter() -> impl Iterator<Item=(&'static OsStr, &'static OsStr)>{
    let cccache_params_iter = {
        [
            ("CCACHE_MAXSIZE", "50G"),
            ("CCACHE_CPP2", "true"),
            ("CCACHE_HARDLINK", "true"),
            ("CCACHE_SLOPPINESS", "file_macro,time_macros,include_file_mtime,include_file_ctime,file_stat_matches")
        ]
        .iter()
        .map(|val|{
            (std::ffi::OsStr::new(val.0), std::ffi::OsStr::new(val.1))
        })
    };
    cccache_params_iter
}

fn spawn_compiler() -> Result<Child, io::Error> {
    let distcc_path: &Path = Path::new("/usr/local/bin/distcc");
    let distcc_hosts_path: &Path = Path::new("~/.distcc/hosts");
    let distcc_pump_path: &Path = Path::new("/usr/local/bin/pump");
    let ccache_path: &Path = Path::new("/usr/local/bin/ccache");

    // Аргументы приложения включая путь к нему
    let mut args = args()
        .skip(1); // Пропускаем имя самого приложения

    // Путь к компилятору
    // TODO: Убрать как-то PathBuf?
    let clang_path: PathBuf = {
        let path = args
            .next()
            .expect("Missing compiler argument");

        PathBuf::from(path)
    };
    assert!(clang_path.exists(), "Clang doesn't exist at path"); // TODO: Print path
    //dbg!(&clang_path);

    // Аргументы компилятора
    let compiler_args_iter = args
        .into_iter();

    // Итератор по параметрам окружения ccache
    // let cccache_params_iter = get_cccache_params_iter();

    // Флаги наличия бинарников distcc
    let distcc_exists = distcc_path.exists();
    let distcc_pump_exists = distcc_pump_path.exists();
    let dist_cc_hosts_exist = if distcc_exists {
        file_is_not_empty_and_exists(distcc_hosts_path)
    }else{
        false
    };

    // Флаги наличия бинарника ccache
    let ccache_exists = ccache_path.exists();

    // Выбираем, что именно исполнять
    let command_result = if distcc_exists && distcc_pump_exists && dist_cc_hosts_exist && ccache_exists {
        // CCCache + DistCC + DistCC-Pump
        //println!("CCCache + DistCC + DistCC-Pump");
        Command::new(distcc_pump_path)
            .envs(build_cccache_params_iter())
            .env("CCACHE_PREFIX", distcc_path)
            .arg(ccache_path)
            .arg(clang_path)
            .args(compiler_args_iter)
            .spawn()
    }else if distcc_exists && dist_cc_hosts_exist && ccache_exists {
        // CCCache + DistCC
        //println!("CCCache + DistCC");
        Command::new(ccache_path)
            .envs(build_cccache_params_iter())
            .env("CCACHE_PREFIX", distcc_path)
            .arg(clang_path)
            .args(compiler_args_iter)
            .spawn()
    }else if ccache_exists{
        // CCCache
        //println!("CCCache");
        Command::new(ccache_path)
            .envs(build_cccache_params_iter())
            .arg(clang_path)
            .args(compiler_args_iter)
            .spawn()
    }else{
        // Compiler only
        //println!("Compiler only");
        Command::new(clang_path)
            .args(compiler_args_iter)
            .spawn()
    };

    command_result
}

// TODO: Возвращать код ошибки компилятора
// https://www.joshmcguigan.com/blog/custom-exit-status-codes-rust/
fn main() {
    // TODO: проверять все возможные хосты
    // $DISTCC_HOSTS
    // $DISTCC_DIR/hosts
    // ~/.distcc/hosts
    // /usr/local/Cellar/distcc/3.3.3_1/etc/distcc/hosts
    
    // TODO: Константы
    /*let distcc_path: &Path = Path::new(env!("WRAPPER_DISTCC_PATH"));
    let distcc_hosts_path: &Path = Path::new(env!("WRAPPER_DISTCC_HOSTS_PATH"));
    // let distcc_pump_path: &Path = Path::new("/usr/local/bin/pump");
    let ccache_path: &Path = Path::new(env!("WRAPPER_CCCACHE_PATH"));
    let clang_path: &Path = Path::new(env!("WRAPPER_COMPILER_PATH"));
    
    assert!(!distcc_path.as_os_str().is_empty(), "Empty distcc_path");
    assert!(!distcc_hosts_path.as_os_str().is_empty(), "Empty distcc_hosts_path path");
    assert!(!ccache_path.as_os_str().is_empty(), "Empty ccache_path");
    assert!(!clang_path.as_os_str().is_empty(), "Empty clang_path");*/
    
    let command_result = spawn_compiler();

    // Результат работа комманды
    let child_wait_res = {
        // Проверяем валидность спавна процесса
        let mut child_process = match command_result {
            Ok(res) => res,
            Err(e) => panic!("Command spawn error: {}", e)
        };
        // Ждем результата
        match child_process.wait(){
            Ok(res) => res,
            Err(e) => panic!("Command wait error: {}", e)
        }
    };

    let compiler_exit_code = child_wait_res
        .code()
        .expect("Compiler exit code does not exist");

    // Выдаем наружу код возврата подпроцесса
    std::process::exit(compiler_exit_code);
}