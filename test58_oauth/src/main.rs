mod error;
mod facebook_env_params;
mod app_middlewares;
mod constants;

use actix_files::{
    Files
};
use actix_web::{
    web::{
        self
    },
    guard::{
        self
    },
    HttpServer,
    App
};
use handlebars::{
    Handlebars
};
use rand::{
    Rng
};
use actix_identity::{
    CookieIdentityPolicy, 
    IdentityService
};
use crate::{
    error::{
        AppError
    },
    facebook_env_params::{
        FacebookEnvParams
    },
    app_middlewares::{
        create_error_middleware,
        create_check_login_middleware
    }
};

async fn index(handlebars: web::Data<Handlebars<'_>>) -> Result<web::HttpResponse, AppError> {
    let body = handlebars.render(constants::INDEX_TEMPLATE, &serde_json::json!({}))?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
}

async fn login_page(handlebars: web::Data<Handlebars<'_>>) -> Result<web::HttpResponse, AppError> {
    let body = handlebars.render(constants::LOGIN_TEMPLATE, &serde_json::json!({}))?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
}

/// Данный метод вызывается при нажатии на кнопку логина в Facebook
async fn login_with_facebook(req: actix_web::HttpRequest, fb_params: web::Data<FacebookEnvParams>) -> Result<web::HttpResponse, AppError> {
    // Адрес нашего сайта + адрес коллбека
    let callback_site_address = {
        let scheme = req
            .uri()
            .scheme_str()
            .ok_or(AppError::ActixError(actix_web::error::ErrorBadRequest("Request authority is missing")))?;

        let authority = req
            .uri()
            .authority()
            .ok_or(AppError::ActixError(actix_web::error::ErrorBadRequest("Request authority is missing")))?;
        format!("{}://{}/facebook/auth_callback", scheme, authority)
    };

    // Создаем урл, на который надо будет редиректиться браузеру для логина
    // https://www.facebook.com/dialog/oauth\
    //      ?client_id=578516362116657\
    //      &redirect_uri=http://localhost/facebook-auth\
    //      &response_type=code\
    //      &scope=email,user_birthday
    let mut redirect_url = url::Url::parse("https://www.facebook.com/dialog/oauth")?;
    redirect_url
        .query_pairs_mut()
        .append_pair("client_id", &fb_params.client_id)
        .append_pair("redirect_uri", &callback_site_address)
        .append_pair("response_type", "code")
        .append_pair("scope", "email,user_birthday")
        .finish();

    // Возвращаем код 302 и Location в заголовках для перехода
    Ok(web::HttpResponse::Found()
        .header(actix_web::http::header::LOCATION, redirect_url.as_str())
        .finish())
}

async fn facebook_auth() -> Result<web::HttpResponse, AppError> {
    Ok(web::HttpResponse::Ok().finish())
}

/// Функция непосредственного конфигурирования приложения
/// Для каждого потока исполнения будет создано свое приложение
fn configure_new_app(config: &mut web::ServiceConfig) {
    config
        .service(web::resource(constants::INDEX_PATH)
                    .wrap(create_check_login_middleware())
                    .route(web::route()
                            .guard(guard::Get())
                            .to(index)))
        .service(web::resource(constants::LOGIN_PATH)
                        .wrap(create_check_login_middleware())
                        .route(web::route()
                                .guard(guard::Get())
                                .to(login_page)))                         
        .service(web::scope("/facebook")
                    .service(web::resource("/login")
                                .route(web::route()
                                        .guard(guard::Post())
                                        .to(login_with_facebook)))
                    .service(web::resource("/auth_callback")
                                .route(web::route()
                                        .guard(guard::Post())
                                        .to(facebook_auth))))
        .service(Files::new("static/css", "static/css"))
        .service(Files::new("static/js", "static/js"));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Получаем параметры Facebook
    let facebook_env_params = web::Data::new(FacebookEnvParams::get_from_env());

    // Создаем шареную ссылку на обработчик шаблонов
    // Пример работы с шаблонами
    // https://github.com/actix/examples/tree/master/template_engines/handlebars
    let handlebars = {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_file(constants::INDEX_TEMPLATE, "templates/index.hbs")
            .unwrap();
        handlebars
            .register_template_file(constants::LOGIN_TEMPLATE, "templates/login.hbs")
            .unwrap();
        handlebars
            .register_template_file(constants::ERROR_TEMPLATE, "templates/error.hbs")
            .unwrap();        
        web::Data::new(handlebars)
    };

    // Ключ для шифрования кук, генерируется каждый раз при запуске сервера
    let private_key = rand::thread_rng().gen::<[u8; 32]>();

    HttpServer::new(move ||{
            // Настраиваем middleware идентификации пользователя, делает зашифрованную куку у пользователя в браузере,
            // тем самым давая возможность проверять залогинен ли пользователь или нет
            let identity_middleware = {
                let policy = CookieIdentityPolicy::new(&private_key)
                    .name("auth-logic")
                    .max_age(60 * 60 * 24 * 30) // 30 дней максимум
                    .secure(false);
                IdentityService::new(policy)
            };

            // Приложение создается для каждого потока свое собственное
            App::new()
                .wrap(create_error_middleware())
                .wrap(identity_middleware)
                .wrap(actix_web::middleware::Logger::default())
                .app_data(handlebars.clone())
                .app_data(facebook_env_params.clone())
                .configure(configure_new_app)
        }) 
        .bind("127.0.0.1:8080")?
        .run()
        .await
}