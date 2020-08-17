use std::{
    io::{
        self
    }
};

#[derive(Debug)]
pub enum CameraImageError {
    ApplicationNotFound,
    TempFilePathError,
    CameraStartFailed,
    CameraCaptureFailed,
    TempFileReadError(io::Error),
}

// TODO:
// impl std::fmt::Display for CameraImageError{
//     fn display
// }

/*impl From<io::Error> for CameraImageError {
    fn from(err: io::Error) -> Self {
        CameraImageError::DeviceNotFound(err)
    }
}*/