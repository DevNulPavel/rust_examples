// use telegram_bot::{
    // Error as TelegramError
// };
use currencies_request::{
    CurrencyError,
    error_from
};

#[derive(Debug)]
pub enum TelegramBotError{
    TelegramErr(telegram_bot::Error),
    CurrencyErr(CurrencyError),
    CustomError(String),
}

error_from!(TelegramBotError, TelegramErr, telegram_bot::Error);
error_from!(TelegramBotError, CurrencyErr, CurrencyError);
error_from!(TelegramBotError, CustomError, &str, to_string);

////////////////////////////////////////////////////////////////////////

pub type TelegramBotResult = Result<(), TelegramBotError>;