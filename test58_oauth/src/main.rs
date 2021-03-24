mod error;
mod facebook_env_params;

use actix_files::{
    Files
};
use actix_web::{
    HttpServer,
    App,
    Responder,
    web::{
        self
    },
    guard::{
        self
    }
};
use handlebars::{
    Handlebars
};
use crate::{
    error::{
        AppError
    },
    facebook_env_params::{
        FacebookEnvParams
    }
};

#[actix_web::get("/")]
async fn index(handlebars: web::Data<Handlebars<'_>>) -> Result<web::HttpResponse, AppError> {
    let body = handlebars.render("index", &serde_json::json!({}))?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
}

#[actix_web::post("/facebook_auth")]
async fn facebook_auth() -> Result<web::HttpResponse, AppError> {
    Ok(web::HttpResponse::Ok().finish())
}

/// Функция непосредственного конфигурирования приложения
/// Для каждого потока исполнения будет создано свое приложение
fn configure_new_app(config: &mut web::ServiceConfig) {
    config
        .service(index)
        .service(facebook_auth)
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
            .register_template_file("index", "html/index.hbs")
            .unwrap();
        web::Data::new(handlebars)
    };

    HttpServer::new(move ||{
        // Приложение создается для каждого потока свое собственное
        App::new()
            .app_data(handlebars.clone())
            .app_data(facebook_env_params.clone())
            .configure(configure_new_app)
    }) 
    .bind("127.0.0.1:8080")?
    .run()
    .await
}