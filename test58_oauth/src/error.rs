use quick_error::{
    quick_error
};
use actix_web::{
    ResponseError
};
use crate::{
    responses::{
        FacebookErrorResponse
    }
};

quick_error!{
    #[derive(Debug)]
    pub enum AppError{
        /// Не смогли отрендерить шаблон
        TemplateRenderError(err: handlebars::RenderError){
            from()
        }

        /// Не смогли отрендерить шаблон
        ActixError(err: actix_web::Error){
            from()
        }

        /// Ошибка парсинга адреса
        URLParseError(err: url::ParseError){
            from()
        }

        /// Ошибка у внутреннего запроса с сервера на какое-то API
        InternalReqwestLibraryError(err: reqwest::Error){
            from()
        }

        /// Сервер Facebook ответил ошибкой какой-то
        FacebookApiError(err: FacebookErrorResponse){
            from()
        }

        /// Произошла ошибка работы с базой данных
        DatabaseError(err: sqlx::Error){
            from()
        }

        /// Ошибка с произвольным описанием
        Custom(err: String){
        }
    }
}

// Для нашего enum ошибки реализуем конвертацию в ResponseError
impl ResponseError for AppError {
}