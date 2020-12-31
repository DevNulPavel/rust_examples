use qrcode::{
    types::{
        QrError
    }
};
use image::{
    ImageError
};

#[derive(Debug)]
pub enum QrCodeError{
    GenerateErr(QrError),
    ImageConvertErr(ImageError)
}

impl From<QrError> for QrCodeError {
    fn from(e: QrError) -> Self {
        QrCodeError::GenerateErr(e)
    }
}
impl From<ImageError> for QrCodeError {
    fn from(e: ImageError) -> Self {
        QrCodeError::ImageConvertErr(e)
    }
}