use std::path::PathBuf;

#[derive(Debug)]
pub enum JsonType {
    Raw,
    Encoded
}

#[derive(Debug)]
pub struct JsonInfo {
    pub path: PathBuf,
    pub json_type: JsonType
}