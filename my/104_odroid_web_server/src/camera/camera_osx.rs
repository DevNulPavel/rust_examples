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
use super::{
    error::{
        CameraImageError,
        CameraCountError
    }
};

// TODO: Сделать для OSX
pub fn get_cameras_count() -> Result<usize, CameraCountError>{
    return Ok(1);
}

pub fn get_camera_image(_: i8) -> Result<Vec<u8>, CameraImageError>{
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
                    std::str::from_utf8(&output.stdout)
                        .map(|str|{
                            PathBuf::from(str.trim_end())
                        })
                        .map_err(|_|{
                            CameraImageError::ApplicationNotFound
                        })?
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

    // TODO: Suppress out
    let ffmpeg_spawn = Command::new(ffmpeg_path)
        .args(&["-f", "avfoundation", 
                //"-video_size", "1280x720", 
                //"-framerate", "30", 
                "-framerate", "5",
                "-i", "0", 
                "-vframes", "1",
                temporary_file_path_str])
        .spawn();
    
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