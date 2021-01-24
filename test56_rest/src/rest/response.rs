use serde::{
    Serialize
};

#[cfg(test)]
use serde::{
    Deserialize
};

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize))]
pub struct UploadImageResponseData{
    pub base_64_preview: String,
}

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize))]
pub struct UploadImageResponse{
    pub images: Vec<UploadImageResponseData>
}

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize))]
pub struct UploadImageError{
    pub message: String
}