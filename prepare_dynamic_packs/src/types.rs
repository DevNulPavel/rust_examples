use std::path::{PathBuf, Path};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PackConfig{
    pub name: String,
    pub resources: Vec<String>,
    pub priority: i32,
    pub required: bool
}

#[derive(Debug)]
pub struct PackFilePathInfo{
    pub absolute: PathBuf,
    pub relative: String
}

#[derive(Debug)]
pub struct PackData {
    pub pack_name: String,
    pub files_paths: Vec<PackFilePathInfo>,
    pub priority: i32,
    pub required: bool,
    pub pack_size: u64
}