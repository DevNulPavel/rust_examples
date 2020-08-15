mod camera;
mod error;

pub use self::{
    camera::{
        get_camera_image
    },
    error::{
        CameraImageError
    }
};