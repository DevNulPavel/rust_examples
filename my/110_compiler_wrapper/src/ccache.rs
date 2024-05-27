use std::{
    path::{
        Path,
        PathBuf
    }
};
use crate::{
    helpers::{
        is_env_var_enabled,
        get_executable_full_path
    }
};

pub struct CCCachePaths{ 
    ccache_path: Option<PathBuf>
}

impl CCCachePaths{
    pub fn new() -> CCCachePaths{
        // Пути к исполняемым файлам
        let path: Option<PathBuf> = match get_executable_full_path("ccache") {
            Some(path) => Some(PathBuf::from(&path)),
            None => None
        };
        CCCachePaths{
            ccache_path: path
        }
    }

    pub fn can_use_ccache<'a>(&'a self) -> Option<&'a Path>  {
        match self.ccache_path {
            Some(ref ccache_path) => {
                let allow = ccache_path.exists() 
                    && is_env_var_enabled("XGEN_ENABLE_CCACHE");
                if allow {
                    Some(&ccache_path)
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
