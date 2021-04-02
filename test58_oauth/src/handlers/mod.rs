mod http_handlers;
mod auth_handlers;

use actix_files::{
    Files
};
use actix_web::{
    guard::{
        self
    }, 
    web::{
        self
    }
};
use crate::{
    app_middlewares::{
        create_user_info_middleware,
        create_auth_check_middleware
    },
    constants::{
        self
    }
};

/// Функция непосредственного конфигурирования приложения
/// Для каждого потока исполнения будет создано свое приложение
pub fn retup_routes(config: &mut web::ServiceConfig) {
    config
        .service(web::resource(constants::INDEX_PATH)
                    .wrap(create_user_info_middleware(
                            || {
                                web::HttpResponse::Found()
                                    .header(actix_web::http::header::LOCATION, constants::LOGIN_PATH)
                                    .finish()
                            }))
                    .route(web::route()
                            .guard(guard::Get())
                            .to(http_handlers::index)))
        .service(web::resource(constants::LOGIN_PATH)
                    .wrap(create_auth_check_middleware(
                            false,
                            || {
                                web::HttpResponse::Found()
                                    .header(actix_web::http::header::LOCATION, constants::INDEX_PATH)
                                    .finish()
                            }))
                    .route(web::route()
                                .guard(guard::Get())
                                .to(http_handlers::login_page)))                         
        .service(web::resource(constants::LOGOUT_PATH)
                    .route(web::route()
                                .guard(guard::Post())
                                .to(http_handlers::logout))) 
        .service(web::scope(constants::FACEBOOK_SCOPE_PATH)
                    .service(web::resource(constants::LOGIN_PATH)
                                .route(web::route()
                                        .guard(guard::Post())
                                        .to(auth_handlers::login_with_facebook)))
                    .service(web::resource(constants::AUTH_CALLBACK_PATH)
                                .route(web::route()
                                        .guard(guard::Get())
                                        .to(auth_handlers::facebook_auth_callback))))
        .service(web::scope(constants::GOOGLE_SCOPE_PATH)
                    .service(web::resource(constants::LOGIN_PATH)
                                .route(web::route()
                                        .guard(guard::Post())
                                        .to(auth_handlers::login_with_google)))
                    .service(web::resource(constants::AUTH_CALLBACK_PATH)
                                .route(web::route()
                                        .guard(guard::Get())
                                        .to(auth_handlers::google_auth_callback))))
        .service(Files::new("static/css", "static/css"))
        .service(Files::new("static/js", "static/js"))
        .service(Files::new("static/images", "static/images"));
}