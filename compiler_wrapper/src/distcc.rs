use std::{
    path::{
        Path,
        PathBuf
    }
};
use crate::{
    helpers::{
        file_is_not_empty_and_exists,
        is_env_var_enabled
    }
};


pub struct DistCCPaths{ 
    pub distcc_path: &'static Path
}

impl DistCCPaths{
    pub fn new() -> DistCCPaths{
        // Пути к исполняемым файлам
        //let distcc_pump_path: &Path = Path::new("/usr/local/bin/pump");
        //dbg!(&distcc_hosts_path);
        let distcc_path: &Path = Path::new("/usr/local/bin/distcc");
        DistCCPaths{
            distcc_path
        }
    }

    pub fn can_use_distcc(&self) -> bool {
        // TODO: проверять все возможные хосты
        // $DISTCC_HOSTS
        // $DISTCC_DIR/hosts
        // ~/.distcc/hosts
        // /usr/local/Cellar/distcc/3.3.3_1/etc/distcc/hosts

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