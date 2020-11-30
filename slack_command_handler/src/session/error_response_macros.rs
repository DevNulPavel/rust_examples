

// TODO: Экспорт макроса из модуля, а не для всего крейта
#[macro_export]
macro_rules! slack_response_with_error{
    ($session: expr, $error_text: expr) =>{
        use log::error;
        use crate::session::ResponseWithError;

        error!("{}", $error_text);
        $session.slack_response_with_error($error_text);
    }
}