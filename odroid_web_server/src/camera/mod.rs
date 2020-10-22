// https://doc.rust-lang.org/reference/conditional-compilation.html

mod error;
#[cfg(target_os = "macos")] 
mod camera_osx;
// #[cfg(target_os = "linux")]
mod camera_linux;


pub use self::{
    error::{
        CameraImageError,
        CameraCountError
    }
};
#[cfg(target_os = "macos")]
pub use self::{
    camera_osx::{
        get_camera_image,
        get_cameras_count
    }
};
#[cfg(target_os = "linux")]
pub use self::{
    camera_linux::{
        get_camera_image,
        get_cameras_count
    }
};