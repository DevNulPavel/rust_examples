use quick_error::{
    quick_error
};
use actix_web::{
    http::{
        StatusCode
    },
    dev::{
        HttpResponseBuilder
    },
    HttpResponse,
    ResponseError
};
use serde_json::{
    json
};
use crate::{
    responses::{
        FacebookErrorResponse,
        GoogleErrorResponse
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
        InternalReqwestLibraryError(context: &'static str, err: reqwest::Error){
            context(context: &'static str, err: reqwest::Error) -> (context, err)
        }

        /// Сервер Facebook ответил ошибкой какой-то
        FacebookApiError(err: FacebookErrorResponse){
            from()
        }

        /// Сервер Google ответил ошибкой какой-то
        GoogleApiError(err: GoogleErrorResponse){
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

// Для нашего enum ошибки реализуем конвертацию в ResponseError,
// но делаем это так, чтобы ответ был в виде json
impl ResponseError for AppError {
    // Код ошибки
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
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