mod light_control;
mod error;

pub use {
    light_control::{
        set_light_status
    },
    error::{
        GPIOError
    }
};