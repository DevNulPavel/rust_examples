/*use actix_http::{
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
    // web::{
    //     self
    // }
    // Result
};
// use handlebars::{
//     Handlebars
// };
use serde_json::{
    json
};

/// Функция, которая создает middleware-обработчик ошибки
pub fn create_error_middleware() -> ErrorHandlers<Body> {
    // Для ошибок типа NOT_FOUND назначаем обработчик
    ErrorHandlers::new()
        .handler(StatusCode::NOT_FOUND, not_found_handler)
}

/// Непосредственно обработчик для 404 ошибки
fn not_found_handler<B>(res: ServiceResponse<B>) -> actix_web::Result<ErrorHandlerResponse<B>> {
    let response = get_error_response(&res, "Page not found");
    Ok(ErrorHandlerResponse::Response(res.into_response(response.into_body()),))
}

// Generic error handler.
fn get_error_response<B>(res: &ServiceResponse<B>, error: &str) -> Response<Body> {
    let json_response = json!({
        "error_text": error
    });
    Response::build(res.status())
        .json(json_response)
}*/