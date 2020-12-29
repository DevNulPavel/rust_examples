mod sender_trait;
mod slack_sender;
mod terminal_sender;

pub use self::{
    sender_trait::{
        ResultSender
    },
    slack_sender::{
        SlackResultSender
    },
    terminal_sender::{
        TerminalSender
    }
};