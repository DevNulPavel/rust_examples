use actix_web::{
    web::{
        self,
        ServiceConfig
    },
    Responder,
    //HttpRequest,
    HttpResponse,
    get,
    post    
};
use actix_identity::{
    Identity
};
use serde::{
    Deserialize
};
use crate::{
    constants
};


#[derive(Deserialize)]
struct LoginParams{
    login: String,
    password: String
}

#[get("/")]
async fn index_get(id: Identity) -> impl Responder {
    format!("Hello {}", id.identity().unwrap_or_else(|| {
        "Anonymous".to_owned()
    }))
}

#[get("/login")]
async fn login_get() -> impl Responder {
    // TODO: Переделать на чтение файла, а лучше на кеширование
    let login_page = include_str!("../../static/login_form.html");

    // Страничка логина
    let response = HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(login_page);
    
    response
}

#[post("/login")]
async fn login_post(post_params: web::Form<LoginParams>, id: Identity) -> impl Responder {
    let lowercase_login = post_params.login.to_lowercase();

    let valid_login = {
        lowercase_login == constants::LOGIN
    };
    
    let valid_password = {
        let digest = md5::compute(post_params.password.as_str());
        let password_md5 = format!("{:x}", digest); // TODO: Убрать конвертацию
        password_md5 == constants::PASSWORD_HASH
    };

    if valid_login && valid_password {
        println!("Login remember");
        id.remember(lowercase_login);
    }else{
        println!("Login clean");
        id.forget();
    }

    // После POST запроса на логин - переходим в корень
    HttpResponse::Found()
        .header("location", "/")
        .finish()
}

// TODO: убрать POST
// https://github.com/actix/examples/blob/master/cookie-auth/src/main.rs
async fn logout(id: Identity) -> impl Responder {
    id.forget();

    // После POST запроса на логин - переходим в корень
    HttpResponse::Found()
        .header("location", "/")
        .finish()
}

pub fn configure_http_service(cfg: &mut ServiceConfig){
    cfg
        .service(index_get)
        .service(login_get)
        .service(login_post)
        .service(web::resource("/logout")
                    .route(web::route().to(logout)));
}