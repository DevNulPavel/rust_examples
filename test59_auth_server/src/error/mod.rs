use quick_error::{
    quick_error
};
use serde_json::{
    json
};
use actix_web::{
    http::{
        StatusCode
    },
    error::{
        ResponseError
    },
    dev::{
        HttpResponseBuilder
    },
    HttpResponse
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
        PasswordHashSpawnError(err: actix_web::rt::blocking::BlockingError) {
            from()
        }        

        /// Ошибка у внутреннего запроса с сервера на какое-то API
        ParamValidationError(context: &'static str, err: validator::ValidationErrors){
            context(context: &'static str, err: validator::ValidationErrors) -> (context, err)
        }

        /// Пользователь у нас не авторизован на сервере
        UnautorisedError(info: &'static str){
        }

        /// Ошибка с произвольным описанием
        Custom(err: String){
        }
    }
}

// Для нашего enum ошибки реализуем конвертацию в ResponseError,
// но делаем это так, чтобы ответ был в виде json
impl ResponseError for AppError {
    // Код ошибки
    fn status_code(&self) -> StatusCode {
        match self {
            Self::UnautorisedError(_) => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    // Создаем ответ в виде json
    fn error_response(&self) -> HttpResponse {
        let data = json!({
            "code": self.status_code().as_u16(),
            "message": self.to_string()
        });
        HttpResponseBuilder::new(self.status_code())
            .json(data)
    }    
}