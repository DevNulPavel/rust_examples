#![warn(clippy::all)]

use std::{
    path::{
        Path
    },
    process::{
        Command,
        Child,
        ExitStatus,
    },
    env::{
        args
    },
    io::{
        self
    }
};

// TODO: Возвращать код ошибки компилятора
fn main() -> Result<(), io::Error>{
    // $DISTCC_HOSTS
    // $DISTCC_DIR/hosts
    // ~/.distcc/hosts
    // /usr/local/Cellar/distcc/3.3.3_1/etc/distcc/hosts
    
    // TODO: Константы
    let distcc_path: &Path = Path::new("/usr/local/bin/distcc");
    let distcc_hosts_path: &Path = Path::new("~/.distcc/hosts");
    let distcc_pump_path: &Path = Path::new("/usr/local/bin/pump");
    let ccache_path: &Path = Path::new("/usr/local/bin/ccache");
    let clang_path: &Path = Path::new("/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang");

    let distcc_exists = distcc_path.exists();
    let distcc_hosts_exist = distcc_hosts_path.exists();
    let ccache_exists = ccache_path.exists();

    // TODO: Print path
    assert!(clang_path.exists(), "Clang doesn't exist at path");

    if distcc_exists && ccache_exists {

    }else if ccache_exists{

    }else{
        // TODO: Пропускать первый параметр
        let args_iter = args().into_iter();
        let result = Command::new(clang_path.as_os_str())
            .args(args_iter)
            .spawn();
        let res = result?.wait()?;
        if res.success(){
            return Ok(());
        }else{
            return Err(io::Error::new(io::ErrorKind::Other, "Compiler error"))
        }
    }

    return Err(io::Error::new(io::ErrorKind::Other, "Unknown error"));
}