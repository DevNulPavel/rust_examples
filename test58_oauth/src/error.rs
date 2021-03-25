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
    }
}

// Для нашего enum ошибки реализуем конвертацию в ResponseError
impl ResponseError for AppError {
}