mod client;
mod view_open_response;
mod view;
mod windows;
mod error;

pub use self::{
    client::{
        SlackClient
    },
    error::{
        SlackViewError
    }
};