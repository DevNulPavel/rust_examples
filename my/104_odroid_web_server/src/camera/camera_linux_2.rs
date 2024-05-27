use std::{
    // io::{
    //     self
    // },
    env::{
        self
    },
    fs::{
        self
    },
    path::{
        PathBuf
    },
    process::{
        Command
    }
};
use log::{
    debug,
    error
};
use uuid::{
    Uuid
};
use super::{
    error::{
        CameraImageError
    }
};


// sudo aptitude install libv4l-dev
pub fn get_camera_image() -> Result<Vec<u8>, CameraImageError>{
    // TODO: перебор устройств
    // TODO: Разрешение

    let mut camera = match rscam::new("/dev/video0"){
        Ok(camera) => {
            camera
        },
        Err(error) => {
            return Err(CameraImageError::DeviceNotFound(error))
        }
    };

    let config = rscam::Config {
        interval: (1, 5),           // 5 fps.
        resolution: (1280, 720),
        format: b"MJPG",
        ..Default::default()
    };

    if let Err(err) = camera.start(&config){
        return Err(CameraImageError::CameraStartFailed(err));
    }

    let frame: rscam::Frame = match camera.capture(){
        Ok(frame) => {
            frame
        },
        Err(err) => {
            return Err(CameraImageError::CameraCaptureFailed(err));
        }
    };

    Ok(frame)
}