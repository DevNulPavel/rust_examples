use actix_web::{
    web::{
        self
    },
    guard::{
        self
    },
    HttpResponse
};
use tracing::{
    instrument
};
use sqlx::{
    PgPool
};
use quick_error::{
    ResultExt
};
use validator::{
    Validate
};
use serde::{
    Serialize,
    Deserialize
};
use crate::{
    error::{
        AppError
    },
    models::{
        user::{
            User,
            CreateUserConfig
        }
    },
    crypto::{
        CryptoService
    }
};


#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NewUserReqData {
    #[validate(length(min = 3))]
    pub user_name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 3))]
    pub password: String
}

#[instrument]
async fn signup(data: web::Json<NewUserReqData>, 
                db: web::Data<PgPool>, 
                crypto: web::Data<CryptoService>) -> Result<HttpResponse, AppError> {
    // TODO: Middleware для валидации
    data
        .validate()
        .context("New user data error")?;
    
    // TODO: Рандомная соль

    // TODO: Запись в базу пользователя

    // TODO: отдать в виде json

    Ok(HttpResponse::Ok().finish())
}


#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateProfileReqData {
    pub full_name: Option<String>,
    pub bio: Option<String>,
    #[validate(url)]
    pub image: Option<String>
}

/// Функция непосредственного конфигурирования приложения
/// Для каждого потока исполнения будет создано свое приложение
pub fn configure_routes(config: &mut web::ServiceConfig) {
    config
        .service(web::resource("/signup")
                    .route(web::route()
                            .guard(guard::Post())
                            .to(signup)));

    /*config
        .service(web::resource(constants::INDEX_PATH)
                    .wrap(create_user_info_middleware(
                            || {
                                web::HttpResponse::Found()
                                    .header(actix_web::http::header::LOCATION, constants::LOGIN_PATH)
                                    .finish()
                            }))
                    .route(web::route()
                            .guard(guard::Get())
                            .to(index)))
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
                                .to(login_page)))                         
        .service(web::resource(constants::LOGOUT_PATH)
                    .route(web::route()
                                .guard(guard::Post())
                                .to(logout))) 
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
        .service(Files::new("static/images", "static/images"));*/
}