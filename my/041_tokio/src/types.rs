use std::string::String;
use std::path::PathBuf;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::errors::ProcessError;

#[derive(Debug)]
pub enum ProcessCommand{
    Stop,
    Process(ConverRequest)
}

pub type ConverRequest = (PathBuf, ResultSender);

pub type EmptyResult = Result<(), ProcessError>;
pub type StringResult = Result<String, ProcessError>;

pub type ParseResult = (PathBuf, String);

pub type PathSender = UnboundedSender<ProcessCommand>;
pub type PathReceiver = UnboundedReceiver<ProcessCommand>;
pub type ResultSender = UnboundedSender<ParseResult>;
pub type ResultReceiver = UnboundedReceiver<ParseResult>;

