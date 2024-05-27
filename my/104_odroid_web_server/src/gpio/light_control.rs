
// Doc:
// https://wiki.odroid.com/odroid-c1/application_note/gpio/wiringpi#tab__odroid-c1

// Install:
// sudo apt install software-properties-common
// sudo add-apt-repository ppa:hardkernel/ppa
// sudo apt update
// sudo apt install odroid-wiringpi

// Use:
// sudo gpio readall -a
// sudo gpio mode 0 out
// sudo gpio write 0 0

use std::{
    process::{
        Command
    },
    path::{
        PathBuf
    }
};
use log::{
    error
};
use super::{
    error::{
        GPIOError
    }
};

pub fn set_light_status(enabled: bool, wiring_pin_number: i8) -> Result<(), GPIOError>{
    // TODO: Check SUDO
    Command::new("id")
        .arg("-u")
        .output()
        .map_err(|err|{
            error!("id call error: {}", err);
            GPIOError::SudoRequired
        })
        .and_then(|output|{
            if output.status.success() {
                std::str::from_utf8(&output.stdout)
                    .map_err(|err|{
                        error!("id parse failed: {}", err);
                        GPIOError::SudoRequired
                    })
                    .and_then(|str|{
                        if str.trim_end().eq("0"){
                            Ok(())
                        }else{
                            Err(GPIOError::SudoRequired)
                        }
                    })
            }else{
                error!("id call failed: is not success status");
                Err(GPIOError::SudoRequired)
            }
        })?;

    let gpio_path = Command::new("which")
        .arg("gpio")
        .output()
        .map_err(|err|{
            error!("Which call failed: {}", err);
            GPIOError::WhichExecuteFailed
        })
        .and_then(|output|{
            if output.status.success() {
                std::str::from_utf8(&output.stdout)
                    .map_err(|err|{
                        error!("Which parse failed: {}", err);
                        GPIOError::WhichParseError
                    })
                    .map(|str|{
                        PathBuf::from(str.trim_end())
                    })
            }else{
                error!("Which call failed: is not success status");
                Err(GPIOError::GPIOAppNotFound)
            }
        })?;

    let pin_number_str = format!("{}", wiring_pin_number);

    // TODO: Suppress out
    Command::new(&gpio_path)
        .args(&["mode", pin_number_str.as_str(), "out"])
        .spawn()
        .map_err(|err|{
            error!("GPIO run failed: {}", err);
            GPIOError::GPIORunFailed
        })?
        .wait()
        .map_err(|err|{
            error!("GPIO wait failed: {}", err);
            GPIOError::GPIOWaitFailed
        })
        .and_then(|status|{
            if status.success(){
                Ok(())
            }else{
                error!("GPIO call failed: is not success status");
                Err(GPIOError::GPIOSpawnFailed)
            }        
        })?;
    
    let value_str = if enabled {
        "1"
    }else{
        "0"
    };

    // TODO: Suppress out
    Command::new(gpio_path)
        .args(&["write", pin_number_str.as_str(), value_str])
        .spawn()
        .map_err(|err|{
            error!("GPIO run failed: {}", err);
            GPIOError::GPIORunFailed
        })?
        .wait()
        .map_err(|err|{
            error!("GPIO wait failed: {}", err);
            GPIOError::GPIOWaitFailed
        })
        .and_then(|status|{
            if status.success(){
                Ok(())
            }else{
                error!("GPIO call failed: is not success status");
                Err(GPIOError::GPIOSpawnFailed)
            }        
        })?;

    Ok(())
}