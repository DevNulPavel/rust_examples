use std::{
    fs::{
        self
    }
};
use log::{
    // debug,
    info,
    error
};
use actix_web::{
    web::{
        self,
        ServiceConfig
    },
    Responder,
    //HttpRequest,
    HttpResponse, 
};
use actix_identity::{
    Identity
};
use serde::{
    Deserialize,
    Serialize
};
use crate::{
    middlewares::{
        check_login::{
            CheckLogin
        }
    },
    camera::{
        get_camera_image,
        get_cameras_count
    },
    gpio::{
        set_light_status
    },
    constants
};


#[derive(Deserialize)]
struct LoginParams{
    login: String,
    password: String
}

async fn index_get() -> impl Responder {
    info!("Index request");

    // TODO: Кеширование
    let data = match fs::read("html/index.html") {
        Ok(data) => {
            data
        },
        Err(err) => {
            error!("Index file read failed: {}", err);
            return HttpResponse::NoContent()
                .body("No file");
        }
    };

    // Страничка логина
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(data)
}

async fn cameras_count_get() -> impl Responder {
    info!("Cameras count request");

    #[derive(Serialize)]
    struct Response{
        count: usize
    }

    match get_cameras_count(){
        Ok(count) => {
            HttpResponse::Ok()
                .json(Response{
                    count
                })
        },
        Err(err) => {
            error!("Camera count error: {:?}", err);
            HttpResponse::NoContent()
                .finish()
        }
    }
}

#[derive(Debug, Deserialize)]
struct ImageRequestParams{
    camera_index: i8
}

async fn image_from_camera_get(params: web::Query<ImageRequestParams>) -> impl Responder {
    info!("Image request");

    /*let data = match fs::read("images/test.jpg") {
        Ok(data) => {
            data
        },
        Err(err) => {
            error!("Image read failed: {}", err);
            return HttpResponse::NoContent()
                .body("No file");
        }
    };*/
    match get_camera_image(params.camera_index){
        Ok(image) => {
            HttpResponse::Ok()
                .content_type("image/jpeg")
                .body(image)
        },
        Err(err) => {
            error!("Camera image error: {:?}", err);
            HttpResponse::NoContent()
                .finish()
        }
    }
}

async fn image_get() -> impl Responder {
    info!("Image page get request");

    // TODO: fs::NamedFile https://github.com/actix/examples/blob/master/basics/src/main.rs
    // TODO: Кеширование
    let data = match fs::read("html/image.html") {
        Ok(data) => {
            data
        },
        Err(err) => {
            error!("Login form read failed: {}", err);
            return HttpResponse::NoContent()
                .body("No file");
        }
    };

    // Страничка логина
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(data)
}

#[derive(Deserialize)]
struct LightParams{
    status: bool
}

async fn light_post(post_params: web::Json<LightParams>) -> impl Responder {
    info!("Change light mode request");
    match set_light_status(post_params.status, 0){
        Ok(_) => {
            HttpResponse::Ok()
                .finish()
        },
        Err(err) => {
            error!("Change light mode request failed: {:?}", err);
            HttpResponse::NoContent()
                .finish()
        } 
    }
}

async fn login_get() -> impl Responder {
    info!("Login page get request");

    // TODO: Переделать на чтение файла, а лучше на кеширование
    //let login_page = include_str!("../../html/login_form.html");

    // TODO: Кеширование
    let data = match fs::read("html/login_form.html") {
        Ok(data) => {
            data
        },
        Err(err) => {
            error!("Login form read failed: {}", err);
            return HttpResponse::NoContent()
                .body("No file");
        }
    };

    // Страничка логина
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(data)
}

async fn login_post(post_params: web::Form<LoginParams>, id: Identity) -> impl Responder {
    info!("Login page post request");

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
    // TODO: Объединение
    cfg
        .service(web::resource("/")
                    .wrap(CheckLogin::default())
                    .route(web::get().to(index_get)))
        .service(web::resource("/logout")
                    .wrap(CheckLogin::default())
                    .route(web::route().to(logout)))
        .service(web::resource("/cameras_count")
                    .wrap(CheckLogin::default())
                    .route(web::get().to(cameras_count_get)))
        .service(web::resource("/image_from_camera")
                    .wrap(CheckLogin::default())
                    .route(web::get().to(image_from_camera_get)))
        .service(web::resource("/image")
                    .wrap(CheckLogin::default())
                    .route(web::get().to(image_get)))
        .service(web::resource("/light")
                    .wrap(CheckLogin::default())
                    .route(web::post().to(light_post)))
        .service(web::resource("/login")
                    .route(web::get().to(login_get))
                    .route(web::post().to(login_post)));
}