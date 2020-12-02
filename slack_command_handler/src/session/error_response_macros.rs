

// TODO: Экспорт макроса из модуля, а не для всего крейта
#[macro_export]
macro_rules! slack_response_with_error{
    ($session: expr, $error_text: expr) => {
        {
            use log::error;
            use crate::session::ResponseWithError;

            error!("{}", $error_text);
            $session.slack_response_with_error($error_text);
        }
    }
}

#[macro_export]
macro_rules! unwrap_error_with_slack_response_and_return{
    ($value: expr, $session: expr, $error_format: expr) =>{
        match $value {
            Ok(valid_value) => valid_value,
            Err(err) => {
                $crate::slack_response_with_error!($session, format!($error_format, err));
                return;
            }
        }
    }
}

#[macro_export]
macro_rules! unwrap_option_with_slack_response_and_return{
    ($value: expr, $session: expr, $error_text: expr) =>{
        match $value {
            Some(valid_value) => valid_value,
            None => {
                $crate::slack_response_with_error!($session, $error_text.to_owned());
                return;
            }
        }
    }
}