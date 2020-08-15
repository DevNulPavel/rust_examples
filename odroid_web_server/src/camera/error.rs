use std::{
    io::{
        self
    }
};

#[derive(Debug)]
pub enum CameraImageError {
    DeviceNotFound(io::Error),
    CameraStartFailed(rscam::Error),
    CameraCaptureFailed(io::Error)
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