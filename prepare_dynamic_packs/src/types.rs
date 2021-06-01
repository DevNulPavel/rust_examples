use std::path::PathBuf;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PackConfig{
    pub name: String,
    pub resources: Vec<String>,
    pub priority: i32,
    pub required: bool
}

#[derive(Debug, Deserialize)]
pub struct PackData{
    pub pack_name: String,
    pub files: Vec<PathBuf>,
    pub priority: i32,
    pub required: bool,
    pub pack_size: u64
}