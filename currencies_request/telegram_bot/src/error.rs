use std::{
    fmt::{
        Display,
        Formatter
    }
};
use currency_lib::{
    error_from
};

// Реализация Display для кастомных ошибок
#[derive(Debug)]
pub enum TelegramBotError{
    TelegramErr(telegram_bot::Error),
    CurrencyErr(currency_lib::CurrencyError),
    DatabaseErr{
        err: sqlx::Error,
        context: DatabaseErrKind
    },
    CustomError(String),
}

error_from!(TelegramBotError, TelegramErr, telegram_bot::Error);
error_from!(TelegramBotError, CurrencyErr, currency_lib::CurrencyError);
error_from!(TelegramBotError, CustomError, &str, to_string);

impl Display for TelegramBotError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TelegramBotError::TelegramErr(e) => {
                e.fmt(f)
            },
            TelegramBotError::CurrencyErr(e) => {
                e.fmt(f)
            },
            TelegramBotError::DatabaseErr{err, context} => {
                match context {
                    DatabaseErrKind::InsertUser => {
                        write!(f, "Insert user error: {}", err)
                    },
                    DatabaseErrKind::RemoveUser => {
                        write!(f, "Remove user error: {}", err)
                    },
                    /*DatabaseErrKind::LoadUser => {
                        write!(f, "Load user error: {}", err)
                    },*/
                    DatabaseErrKind::Unknown => {
                        err.fmt(f)
                    }
                }
            },
            TelegramBotError::CustomError(e) => {
                write!(f, "Unknown error: {}", e)
            },
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl From<sqlx::Error> for TelegramBotError{
    fn from(e: sqlx::Error) -> Self {
        TelegramBotError::DatabaseErr{
            err: e,
            context: DatabaseErrKind::Unknown
        }
    }
}

#[derive(Debug)]
pub enum DatabaseErrKind{
    Unknown,
    InsertUser,
    RemoveUser,
    //LoadUser
}

////////////////////////////////////////////////////////////////////////

pub type TelegramBotResult = Result<(), TelegramBotError>;