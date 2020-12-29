use std::{fmt::{
        Formatter,
        Display,
        self
    }, writeln};

#[derive(Debug)]
pub struct UploadResult{
    pub message: Option<String>,
    pub download_url: Option<String>,
    pub install_url: Option<String>
}

impl Display for UploadResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:#?}", self)
    }
}