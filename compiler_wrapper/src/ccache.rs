use std::{
    path::{
        Path
    }
};
use crate::{
    helpers::{
        is_env_var_enabled
    }
};

pub struct CCCachePaths{ 
    pub ccache_path: &'static Path
}

impl CCCachePaths{
    pub fn new() -> CCCachePaths{
        // Пути к исполняемым файлам
        let ccache_path: &Path = Path::new("/usr/local/bin/ccache");
        CCCachePaths{
            ccache_path
        }
    }

    pub fn can_use_ccache(&self) -> bool {
        self.ccache_path.exists()
            && is_env_var_enabled("XGEN_ENABLE_CCACHE")
    }
}
