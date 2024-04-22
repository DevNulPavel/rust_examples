#![macro_use]

/*// #[macro_export]
macro_rules! unwrap_or_fail_resp {
    ($code: expr) => {
        match $code {
            Ok(val) => val,
            Err(err) => {
                // Выводим ошибку
                error!("{}", err);

                // Создаем ответ с правильным статусом
                let resp = response_with_status_and_empty_body(StatusCode::INTERNAL_SERVER_ERROR);

                // Выходим с ошибкой
                return Ok(resp);
            }
        }
    };
    ($code: expr, $err_status: expr) => {
        match $code {
            Ok(val) => val,
            Err(err) => {
                // Выводим ошибку
                error!("{}", err);

                // Создаем ответ с правильным статусом
                let resp = response_with_status_and_empty_body($err_status);

                // Выходим с ошибкой
                return Ok(resp);
            }
        }
    };
}*/

// #[macro_export]
/*macro_rules! true_or_fail_resp {
    ($code: expr, $err_status: expr, $desc: literal) => {
        if !$code {
            error!($desc);
            return Ok(response_with_status_end_error($err_status, $desc));
        }
    };
}*/
