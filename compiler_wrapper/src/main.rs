#![warn(clippy::all)]
// #![feature(lang_items)]
// #![feature(start)]
// #![no_std]

use std::{
    path::{
        Path,
        PathBuf
    },
    process::{
        Command,
        //Child,
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
    
    let distcc_path: &Path = Path::new("/usr/local/bin/distcc");
    let distcc_hosts_path: &Path = Path::new("~/.distcc/hosts");
    //let distcc_pump_path: &Path = Path::new("/usr/local/bin/pump");
    let ccache_path: &Path = Path::new("/usr/local/bin/ccache");

    // Флаги наличия бинарника
    let distcc_exists = distcc_path.exists();
    let ccache_exists = ccache_path.exists();

    // Путь к компилятору
    let clang_path: PathBuf = {
        let path = args()
            .skip(1)
            .next()
            .expect("Missing compiler argument");

        PathBuf::from(path)
    };
    assert!(clang_path.exists(), "Clang doesn't exist at path"); // TODO: Print path
    
    //dbg!(&clang_path);

    // Аргументы компилятора
    let compiler_args_iter = args()
        .into_iter()
        .skip(2); // TODO: Пропускать первый параметр?

    // Итератор по параметрам окружения ccache
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

    if distcc_exists && ccache_exists && file_is_not_empty_and_exists(distcc_hosts_path) {
        let result = Command::new(ccache_path.as_os_str())
            .envs(cccache_params_iter)
            .env("CCACHE_PREFIX", distcc_path)
            .arg(clang_path)
            .args(compiler_args_iter)
            .spawn();
        
        let res = result
            .expect("TODO") // TODO: ???
            .wait()
            .expect("TODO");

        if res.success(){
            //return Ok(());
        }else{
            //return Err(io::Error::new(io::ErrorKind::Other, "Compiler error"))
            std::process::exit(res.code().expect("Compiler exit code does not exist"))
        }

    }else if ccache_exists{
        let result = Command::new(ccache_path.as_os_str())
            .envs(cccache_params_iter)
            .arg(clang_path)
            .args(compiler_args_iter)
            .spawn();
        
        let res = result
            .expect("TODO") // TODO: ???
            .wait()
            .expect("TODO"); // TODO: ???

        if res.success(){
            //return Ok(());
        }else{
            // return Err(io::Error::new(io::ErrorKind::Other, "Compiler error"))
            std::process::exit(res.code().expect("Compiler exit code does not exist"))
        }
    }else{
        let result = Command::new(clang_path.as_os_str())
            .args(compiler_args_iter)
            .spawn();
        
        let res = result
            .expect("TODO") // TODO: ???
            .wait()
            .expect("TODO"); // TODO: ???

        if res.success(){
            //return Ok(());
        }else{
            //return Err(io::Error::new(io::ErrorKind::Other, "Compiler error"))
            std::process::exit(res.code().expect("Compiler exit code does not exist"))
        }
    }

    // return Err(io::Error::new(io::ErrorKind::Other, "Unknown error"));
}