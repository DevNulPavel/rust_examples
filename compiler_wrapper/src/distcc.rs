use std::{
    path::{
        Path,
        PathBuf
    }
};
use crate::{
    helpers::{
        file_is_not_empty_and_exists,
        is_env_var_enabled,
        get_executable_full_path
    }
};


pub struct DistCCPaths{ 
    distcc_path: Option<PathBuf>
}

impl DistCCPaths{
    pub fn new() -> DistCCPaths{
        // Пути к исполняемым файлам
        //let distcc_pump_path: &Path = Path::new("/usr/local/bin/pump");
        //dbg!(&distcc_hosts_path);
        // /usr/local/bin/
        let path: Option<PathBuf> = match get_executable_full_path("distcc") {
            Some(path) => Some(PathBuf::from(&path)),
            None => None
        };
        DistCCPaths{
            distcc_path: path
        }
    }

    pub fn can_use_distcc<'a>(&'a self) -> Option<&'a Path> {
        // TODO: проверять все возможные хосты
        // $DISTCC_HOSTS
        // $DISTCC_DIR/hosts
        // ~/.distcc/hosts
        // /usr/local/Cellar/distcc/3.3.3_1/etc/distcc/hosts
        match self.distcc_path {
            Some(ref distcc_path) => {
                let distcc_hosts_path: PathBuf = {
                    let home_dir = dirs::home_dir()
                        .expect("Failed to get home directory");
                    //dbg!(&home_dir);
                    home_dir.join(".distcc/hosts")
                };
                //dbg!(distcc_path.exists());
                //dbg!(&distcc_hosts_path);
                let allow = distcc_path.exists() 
                    && file_is_not_empty_and_exists(&distcc_hosts_path, 3) 
                    && is_env_var_enabled("XGEN_ENABLE_DISTCC");
                if allow{
                    Some(distcc_path)
                }else{
                    None
                }
            },
            None => {
                None
            }
        }       
    }
}