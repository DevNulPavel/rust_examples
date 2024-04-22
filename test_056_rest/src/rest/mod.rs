mod json;
mod multipart;
mod response;

pub use self::{
    response::{
        UploadImageResponse,
        UploadImageResponseData,
        UploadImageError
    },
    json::{
        upload_image_json
    },
    multipart::{
        upload_image_multipart
    },
};