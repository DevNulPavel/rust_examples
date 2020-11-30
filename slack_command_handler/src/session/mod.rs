mod base_session;
mod command_session;
mod window_session;
mod error_response_trait;
mod error_response_macros;

pub use self::{
    base_session::{
        BaseSession
    },
    command_session::{
        CommandSession
    },
    window_session::{
        WindowSession
    },
    // Нужно использовать макрос
    error_response_trait::{
        ResponseWithError
    }
};