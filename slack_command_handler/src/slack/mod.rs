mod client;
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