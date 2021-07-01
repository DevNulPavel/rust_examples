use crate::regex::Re;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct ConvertConfig {
    pub ignore_dirs: Vec<Re>,
    pub ignore_files: Vec<Re>,
    pub exclude_files_from_build: Vec<Re>,
    pub forced_include_files_in_build: Vec<Re>,
}

pub struct FoundEntry {
    pub full_source_path: PathBuf,
    pub full_target_path: PathBuf,
}
