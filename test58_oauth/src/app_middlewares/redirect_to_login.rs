use actix_http::{
    body::{
        Body
    }, 
    Response
};
use actix_web::{
    dev::{
        ServiceResponse
    },
    http::{
        StatusCode
    },
    middleware::{
        errhandlers::{
            ErrorHandlerResponse, 
            ErrorHandlers
        }
    },
    web::{
        self
    }
    // Result
};
use handlebars::{
    Handlebars
};
use serde_json::{
    json
};

/*
/// Функция, которая создает middleware-обработчик ошибки
pub fn create_redirect_middleware() -> ErrorHandlers<Body> {
    // Для ошибок типа NOT_FOUND назначаем обработчик
    ErrorHandlers::new()
        .handler(StatusCode::UNAUTHORIZED, unauth_handler)
}

/// Непосредственно обработчик для 401 ошибки
fn unauth_handler<B>(res: ServiceResponse<B>) -> actix_web::Result<ErrorHandlerResponse<B>> {
    Ok(ErrorHandlerResponse::Response(
        res.into_response(HttpResponse::Found()
                            .header(http::header::LOCATION, "/login")
                            .finish()
                            .into_body())
    )
}*/
