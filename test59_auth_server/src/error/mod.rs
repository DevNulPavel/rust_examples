use quick_error::{
    quick_error
};
use actix_web::{
    ResponseError
};

quick_error!{
    #[derive(Debug)]
    pub enum AppError{
        /// Не смогли отрендерить шаблон
        ActixError(err: actix_web::Error){
            from()
        }

        /// Ошибка парсинга адреса
        URLParseError(err: url::ParseError){
            from()
        }

        /// Ошибка у внутреннего запроса с сервера на какое-то API
        InternalReqwestLibraryError(context: &'static str, err: reqwest::Error){
            context(context: &'static str, err: reqwest::Error) -> (context, err)
        }

        /// Произошла ошибка работы с базой данных
        DatabaseError(err: sqlx::Error){
            from()
        }

        /// Ошибка при хешировании паролей
        PasswordHashError(err: argon2::Error){
            from()
        }

        /// Ошибка при спавне хешировании паролей в потоке
        PasswordHashSpawnError {
        }

        /// Ошибка с произвольным описанием
        Custom(err: String){
        }
    }
}

// Для нашего enum ошибки реализуем конвертацию в ResponseError
impl ResponseError for AppError {
}