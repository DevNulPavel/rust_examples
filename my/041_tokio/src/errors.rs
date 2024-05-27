use crate::types::ParseResult;
use crate::types::ProcessCommand;

// https://doc.rust-lang.org/rust-by-example/macros/designators.html
// https://doc.rust-lang.org/reference/macros-by-example.html
// block -
// expr - is used for expressions
// ident - is used for variable/function names
// item -
// literal - is used for literal constants
// pat - (pattern)
// path -
// stmt - (statement)
// tt - (token tree)
// ty - (type)
// vis - (visibility qualifier)
macro_rules! error_from {
    ($err_struct: ty, $enum_val: ident, $source_type: ty) => {
        impl From<$source_type> for $err_struct{
            fn from(e: $source_type) -> Self {
                Self::$enum_val(e)
            }
        }
    };
    ($err_struct: ty, $enum_val: ident, $source_type: ty, $convert_expr: ident) => {
        impl From<$source_type> for $err_struct{
            fn from(e: $source_type) -> Self {
                Self::$enum_val(e.$convert_expr())
            }
        }
    };
}


#[derive(Debug)]
pub enum ProcessError{
    IO(std::io::Error),
    UTF8(std::string::FromUtf8Error),
    ResultChannelSend(tokio::sync::mpsc::error::SendError<ParseResult>),
    TaskChannelSend(tokio::sync::mpsc::error::SendError<ProcessCommand>),
    ChannelReceive(std::sync::mpsc::RecvError),
    Custom(std::string::String),
}
error_from!(ProcessError, IO, std::io::Error);
error_from!(ProcessError, Custom, std::string::String);
error_from!(ProcessError, Custom, &str, to_string);
error_from!(ProcessError, UTF8, std::string::FromUtf8Error);
error_from!(ProcessError, ChannelReceive, std::sync::mpsc::RecvError);
error_from!(ProcessError, ResultChannelSend, tokio::sync::mpsc::error::SendError<ParseResult>);
error_from!(ProcessError, TaskChannelSend, tokio::sync::mpsc::error::SendError<ProcessCommand>);
