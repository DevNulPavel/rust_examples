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
use scopeguard::{
    defer
};
use log::{
    debug,
    error
};
use uuid::{
    Uuid
};
use regex::{
    Regex
};
use lazy_static::{
    lazy_static
};
use super::{
    error::{
        CameraImageError,
        CameraCountError
    }
};

// TODO: Сделать для OSX
pub fn get_cameras_count() -> Result<usize, CameraCountError>{
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^video[0-9]$").unwrap();
    }

    let count = std::fs::read_dir("/dev")
        .map_err(|err|{
            CameraCountError::FilesReadError(err)
        })?
        .filter(|file|{
            match file{
                Ok(file) => {
                    match file.file_name().to_str(){
                        Some(path) => {
                            debug!("Test file: {:?}", path);
                            RE.is_match(path)
                        },
                        None => {
                            false
                        }
                    }
                },
                Err(_) => {
                    false
                }
            }
        })
        .count();

    debug!("Cameras count: {}", count);

    Ok(count)
}

// TODO: Блокировка от одновременного запуска
pub fn get_camera_image(camera_index: i8) -> Result<Vec<u8>, CameraImageError>{
    // TODO: Запуск без sudo требует добавления в группу: sudo usermod -a -G video devnul
    // TODO: Выбор устройства видео
    // TODO: Запускать сервер надо только из терминала, так как из VSСode не даются пермишены на доступ к камере
    // TODO: FPS как параметр

    // https://apple.stackexchange.com/questions/326362/how-to-take-photo-using-terminal
    // ffmpeg -f avfoundation -list_devices true -i ""
    // ffmpeg -ss 0.5 -f avfoundation -i "0" -t 1 capture.jpg
    // 
    // http://iharder.sourceforge.net/current/macosx/imagesnap/

    // ffmpeg -f avfoundation -video_size 1280x720 -framerate 30 -i "0" -vframes 1 out.jpg

    let ffmpeg_path = {
        let ffmpeg_command = Command::new("which")
            .arg("ffmpeg")
            .output();

        match ffmpeg_command {
            Ok(output) => {
                if output.status.success(){
                    match std::str::from_utf8(&output.stdout){
                        Ok(str) => {
                            PathBuf::from(str.trim_end())
                        },
                        Err(_) => {
                            return Err(CameraImageError::ApplicationNotFound);        
                        }
                    }
                }else{
                    return Err(CameraImageError::ApplicationNotFound);
                }
            },
            Err(_) => {
                return Err(CameraImageError::ApplicationNotFound);
            }
        }
    };

    let temporary_file_path = {
        let file_id = Uuid::new_v4();
        env::temp_dir()
            .join(format!("{}.jpg", file_id))
    };

    let temporary_file_path_str = match temporary_file_path.to_str() {
        Some(str) => {
            str
        },
        None => {
            return Err(CameraImageError::TempFilePathError);
        }
    };

    debug!("FFmpeg path: {:?}, Temp file path: {}", ffmpeg_path, temporary_file_path_str);

    let image_device_path = format!("/dev/video{}", camera_index);
    {
        let path = std::path::Path::new(image_device_path.as_str());
        if path.exists() == false {
            return Err(CameraImageError::CameraFileNotFound(image_device_path));
        }
    }

    // TODO: Suppress out
    let ffmpeg_spawn = Command::new(ffmpeg_path)
        .args(&["-f", "video4linux2", 
            "-framerate", "1", 
            "-i", image_device_path.as_str(), 
            "-vframes", "1",
            temporary_file_path_str])
        .spawn();

    drop(image_device_path);
    
    let mut ffmpeg_child_process = match ffmpeg_spawn {
        Ok(child) => {
            child
        },
        Err(_) => {
            return Err(CameraImageError::CameraStartFailed);
        }
    };

    let ffmpeg_exit_status = match ffmpeg_child_process.wait() {
        Ok(exit_status) => {
            exit_status
        },
        Err(err) => {
            error!("FFmpeg capture filed: {}", err);
            return Err(CameraImageError::CameraCaptureFailed);
        }
    };

    if !ffmpeg_exit_status.success() {
        error!("FFmpeg capture filed, exit code: {:?}", ffmpeg_exit_status);
        return Err(CameraImageError::CameraCaptureFailed);
    }

    // Файл будет удален после выхода из функции в деструктора
    defer!{
        fs::remove_file(&temporary_file_path).ok();
    }

    let temporary_file_data = match fs::read(&temporary_file_path){
        Ok(data) => {
            data
        },
        Err(err) => {
            return Err(CameraImageError::TempFileReadError(err));
        }
    };

    Ok(temporary_file_data)
}