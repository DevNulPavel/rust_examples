#![warn(clippy::all)]
// #![feature(lang_items)]
// #![feature(start)]
// #![no_std]

mod distcc;
mod helpers;
mod ccache;

use std::{
    // path::{
        // Path,
        // PathBuf
    // },
    ffi::{
        OsStr
    },
    process::{
        Command,
        Child,
        Output,
    },
    env::{
        args
    },
    io::{
        self,
        Read
    }
};
use crate::{
    distcc::{
        DistCCPaths
    },
    ccache::{
        CCCachePaths
    }
};

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

fn read_compiler_path_file(filename: &str)-> Option<String>{
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
            panic!("Failed to convert {} to utf8", compiler_config_path.to_str().expect("Path to str failed"));
        }
    };

    let trimmed_str = text.trim_end();

    // Создаем переменную пути
    Some(trimmed_str.to_owned())
}

/*fn check_compiler_path(compiler_path: &Path) {
    if !compiler_path.exists() {
        let path = compiler_path
                    .to_str()
                    .expect("Invalid compiler path string");
        panic!("Clang doesn't exist at path: {}", path);
    }
}*/

// <T, I>
//where T: IntoIterator<Item=String> + Sized
/*fn get_compiler(args: &mut std::env::Args) -> String {
    // Путь к компилятору из файлика
    match read_compiler_path_file("compiler_wrapper_compiler_path.cfg"){
        Some(text) => {
            text
        },
        None => {
            // Если нету файлика с компилятором, значит компилятор передан как второй параметр
            let compiler: String = args
                .next()
                .expect("Missing compiler path parameter or 'compiler_wrapper_compiler_path.cfg' file");
            compiler
        }
    }
}*/

fn spawn_compiler() -> Result<Child, io::Error> {
    let mut args = args()
        .skip(1);


    // dbg!(&compiler_path);

    // Проверяем, что компилятор есть по этому пути
    //check_compiler_path(&compiler_path);

    // DistCC
    let distcc = DistCCPaths::new();
    let use_distcc = distcc.can_use_distcc();

    // CCcache
    let ccache = CCCachePaths::new();
    let use_ccache = ccache.can_use_ccache();

    // Содержимое файлика с указанием компилятора
    let compiler_text = match read_compiler_path_file("compiler_wrapper_compiler_path.cfg"){
        Some(text) => {
            text
        },
        None => {
            // Если нету файлика с компилятором, значит компилятор передан как второй параметр
            let compiler: String = args
                .next()
                .expect("Missing compiler path parameter or 'compiler_wrapper_compiler_path.cfg' file");
            compiler
        }
    };

    // Компилятор и наши указанные параметры к нему
    let (compiler, compiler_args_iter) = {
        let mut compiler_args_iter = compiler_text
            .split_whitespace()
            .into_iter();
        let compiler = compiler_args_iter
            .next()
            .expect("Compiler parameter is missing");
        (compiler, compiler_args_iter)
    };

    // Внешние аргументы компилятора
    let wrapper_args_iter = args
        .into_iter();

    // Выбираем, что именно исполнять
    let command_result = match (use_ccache, use_distcc) {
        (Some(ccache_path), Some(distcc_path)) => {
            // CCCache + DistCC
            //println!("CCCache + DistCC");
            Command::new(ccache_path)
                .envs(build_cccache_params_iter())
                .env("CCACHE_PREFIX", distcc_path)
                .arg(compiler)
                .args(compiler_args_iter)
                .args(wrapper_args_iter)
                .spawn()
        }
        (None, Some(distcc_path)) => {
            // DistCC
            //println!("DistCC");
            Command::new(distcc_path)
                .arg(compiler)
                .args(compiler_args_iter)
                .args(wrapper_args_iter)
                .spawn()            
        },
        (Some(ccache_path), None) => {
            // CCCache
            //println!("CCCache");
            Command::new(ccache_path)
                .envs(build_cccache_params_iter())
                .arg(compiler)
                .args(compiler_args_iter)
                .args(wrapper_args_iter)
                .spawn()            
        },
        (None, None) => {
            // Compiler only
            //println!("Compiler only");
            Command::new(compiler)
                .args(compiler_args_iter)
                .args(wrapper_args_iter)
                .spawn()            
        },
    };

    command_result
}

fn get_jobs_count() -> i8 {
    // DistCC
    let distcc = DistCCPaths::new();
    let use_distcc = distcc.can_use_distcc();
    //dbg!(use_distcc);

    match use_distcc {
        Some(path) => {
            let out: Output = Command::new(path)
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
        },
        None => {
            return num_cpus::get() as i8;
        }
    }
}

// Возвращать код ошибки компилятора, но пока не поддерживается stable компилятором
// https://www.joshmcguigan.com/blog/custom-exit-status-codes-rust/
fn main() {   
    // Если нету никаких аргументов, тогда выводим количество потоков сборки
    if args().len() == 1 {
        println!("{}", get_jobs_count());
        return;
    }

    // Результат работа комманды
    let child_wait_res = {
        // Запускам нашу задачу
        let command_result = spawn_compiler();

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
