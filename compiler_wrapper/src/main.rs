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
/*use serde::{    
    Deserialize
};*/

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

fn file_is_not_empty_and_exists(path: impl AsRef<Path>) -> bool {
    let file = match std::fs::File::open(path){
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

/// Итератор по параметрам CCcache
fn build_cccache_params_iter() -> impl Iterator<Item=(&'static OsStr, &'static OsStr)>{
    // https://ccache.dev/manual/3.7.9.html#_configuration_settings
    // ~/.ccache/ccache.conf
    // max_size = 50.0G
    // run_second_cpp = true
    // hard_link = true
    // sloppiness = file_macro,time_macros,include_file_mtime,include_file_ctime,file_stat_matches
    let cccache_params_iter = {
        [
            ("CCACHE_MAXSIZE", "50G"),
            ("CCACHE_CPP2", "true"),        // Должно избавлять от проблем
            ("CCACHE_HARDLINK", "true"),    // Создаются ссылки, вроде бы работает хорошо
            ("CCACHE_SLOPPINESS", "file_macro,time_macros,include_file_mtime,include_file_ctime,file_stat_matches")
        ]
        .iter()
        .map(|val|{
            (std::ffi::OsStr::new(val.0), std::ffi::OsStr::new(val.1))
        })
    };
    cccache_params_iter
}

//#[allow(dead_code)]
fn read_compiler_path_file(filename: &str)-> Option<PathBuf>{
    // TODO: Убрать как-то PathBuf?
    /*let clang_path: PathBuf = {
        let path = args
            .next()
            .expect("Missing compiler argument");

        PathBuf::from(path)
    };*/
    /*let clang_path: &Path = Path::new("/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang");*/

    let current_executable_path = std::env::current_exe()
        .expect("Current executable get path failed");

    let executable_folder = current_executable_path
        .parent()
        .expect("Current executable get directory failed");

    let compiler_config_path = executable_folder.join(filename);

    let mut buf: [u8; 256] = [0; 256];

    // Читаем данные в буффер и получаем длину прочитаннных данных
    let read_len = {
        /*let path_str = compiler_config_path
            .to_str()
            .expect("Invalid compiler config file path");*/
        let mut file = match std::fs::File::open(&compiler_config_path){
            Ok(file) => {
                file
            },
            Err(_) => {
                //panic!("Failed to open compiler config file: {}", path_str);
                return None;
            }
        };
        let len = match file.read(&mut buf){
            Ok(len) => {
                len
            },
            Err(_)=> {
                //panic!("Failed to read compiler config file: {}", path_str);
                return None;
            }
        };
        len
    };

    // Парсим текст из буффера
    let text = match std::str::from_utf8(&buf[0..read_len]) {
        Ok(text) => {
            text 
        },
        Err(_) => {
            panic!("Failed to convert config to utf8");
        }
    };

    let trimmed_str = text.trim_end();

    // Создаем переменную пути
    Some(Path::new(trimmed_str).to_owned())
}

fn check_compiler_path(compiler_path: &Path) {
    if !compiler_path.exists() {
        let path = compiler_path
                    .to_str()
                    .expect("Invalid compiler path string");
        panic!("Clang doesn't exist at path: {}", path);
    }
}

/*#[derive(Deserialize, Debug)]
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

fn is_env_var_enabled(var_name: &str) -> bool {
    match std::env::var(var_name){
        Ok(val) => {
            val.eq("1") || val.eq("true")
        },
        Err(_) => {
            false
        }
    }
}

struct DistCCPaths{ 
    distcc_path: &'static Path
}

impl DistCCPaths{
    fn new() -> DistCCPaths{
        // Пути к исполняемым файлам
        let distcc_path: &Path = Path::new("/usr/local/bin/distcc");
        DistCCPaths{
            distcc_path
        }
    }

    fn can_use_distcc(&self) -> bool {
        let distcc_hosts_path: PathBuf = {
            let home_dir = dirs::home_dir()
                .expect("Failed to get home directory");
            //dbg!(&home_dir);
            home_dir.join(".distcc/hosts")
        };
        self.distcc_path.exists() 
            && file_is_not_empty_and_exists(&distcc_hosts_path) 
            && is_env_var_enabled("XGEN_ENABLE_DISTCC")
    }
}


struct CCCachePaths{ 
    ccache_path: &'static Path
}

impl CCCachePaths{
    fn new() -> CCCachePaths{
        // Пути к исполняемым файлам
        let ccache_path: &Path = Path::new("/usr/local/bin/ccache");
        CCCachePaths{
            ccache_path
        }
    }

    fn can_use_ccache(&self) -> bool {
        self.ccache_path.exists()
            && is_env_var_enabled("XGEN_ENABLE_CCACHE")
    }
}


fn spawn_compiler() -> Result<Child, io::Error> {
    let mut args = args()
        .skip(1);

    // Путь к компилятору
    let compiler_path: PathBuf = match read_compiler_path_file("compiler_wrapper_path.cfg"){
        Some(path) => {
            path
        },
        None => {
            // Если нету файлика с компилятором, значит компилятор передан как второй параметр
            args
                .next()
                .expect("Missing compiler path parameter or 'compiler_wrapper_path.cfg' file")
                .into()
        }
    };
    // dbg!(&compiler_path);

    // Проверяем, что компилятор есть по этому пути
    check_compiler_path(&compiler_path);

    // Пути к исполняемым файлам
    //let distcc_pump_path: &Path = Path::new("/usr/local/bin/pump");
    //dbg!(&distcc_hosts_path);

    // DistCC
    let distcc = DistCCPaths::new();
    let use_distcc = distcc.can_use_distcc();

    // CCcache
    let ccache = CCCachePaths::new();
    let use_ccache = ccache.can_use_ccache();

    // Аргументы компилятора
    let compiler_args_iter = args
        .into_iter();

    /*if distcc_exists && distcc_pump_exists && dist_cc_hosts_exist && ccache_exists {
        // CCCache + DistCC + DistCC-Pump
        //println!("CCCache + DistCC + DistCC-Pump");
        Command::new(distcc_pump_path)
            .envs(build_cccache_params_iter())
            .env("CCACHE_PREFIX", distcc_path)
            .arg(ccache_path)
            .arg(compiler_path)
            .args(compiler_args_iter)
            .spawn()
    }else */

    // Выбираем, что именно исполнять
    let command_result = if use_distcc && use_ccache {
        // CCCache + DistCC
        //println!("CCCache + DistCC");
        Command::new(ccache.ccache_path)
            .envs(build_cccache_params_iter())
            .env("CCACHE_PREFIX", distcc.distcc_path)
            .arg(compiler_path)
            .args(compiler_args_iter)
            .spawn()
    }else if use_distcc {
        // DistCC
        //println!("DistCC");
        Command::new(distcc.distcc_path)
            .arg(compiler_path)
            .args(compiler_args_iter)
            .spawn()
    }else if use_ccache {
        // CCCache
        //println!("CCCache");
        Command::new(ccache.ccache_path)
            .envs(build_cccache_params_iter())
            .arg(compiler_path)
            .args(compiler_args_iter)
            .spawn()
    }else{
        // Compiler only
        //println!("Compiler only");
        Command::new(compiler_path)
            .args(compiler_args_iter)
            .spawn()
    };

    command_result
}

fn get_jobs_count() -> i8 {
    // TODO: expect убрать

    // DistCC
    let distcc = DistCCPaths::new();
    let use_distcc = distcc.can_use_distcc();

    if use_distcc {
        let out: Output = Command::new(distcc.distcc_path)
            .arg("-j")
            .output()
            .expect("Wait failed: distcc -j");
        if out.status.success() {
            let text = std::str::from_utf8(&out.stdout)
                .expect("Out parse failed: distcc -j");
            //dbg!(&text);
            text.trim_end().parse::<i8>()
                .expect("Int parse failed: distcc -j")
        }else{
            panic!("Failed status: distcc -j")
        }
    }else{
        return num_cpus::get() as i8;
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
    
    if args().len() == 1 {
        println!("{}", get_jobs_count());
        return;
    }

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
