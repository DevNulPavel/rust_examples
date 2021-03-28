mod error;
mod auth_handlers;
mod app_middlewares;
mod constants;
mod responses;
mod database;
mod env_app_params;

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
use log::{
    debug
};
use rand::{
    Rng
};
use actix_identity::{
    CookieIdentityPolicy, 
    IdentityService,
    Identity
};
use crate::{
    error::{
        AppError
    },
    env_app_params::{
        FacebookEnvParams,
        GoogleEnvParams
    },
    app_middlewares::{
        create_error_middleware
    },
    database::{
        Database,
        UserInfo
    }
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn get_uuid_from_ident_with_db_check(id: &Identity,
                                          db: &web::Data<Database>) -> Result<Option<String>, AppError>{
    // Проверка идентификатора пользователя
    // TODO: приходится делать это здесь, а не в middleware, так как 
    // есть проблемы с асинхронным запросом к базе в middleware
    if let Some(uuid) = id.identity(){
        // Проверяем, что у нас валидный пользователь из базы
        let exists = db.does_user_uuid_exist(&uuid).await?;
        if !exists {
            // Сброс куки с идентификатором
            id.forget();

            return Ok(None);
        }else{
            return Ok(Some(uuid));
        }
    }else{
        return Ok(None);
    }
}

async fn get_full_user_info_for_identity(id: &Identity,
                                         db: &web::Data<Database>) -> Result<Option<UserInfo>, AppError>{
    // Проверка идентификатора пользователя
    // TODO: приходится делать это здесь, а не в middleware, так как 
    // есть проблемы с асинхронным запросом к базе в middleware
    if let Some(uuid) = id.identity(){
        // Проверяем, что у нас валидный пользователь из базы
        let info = db.try_find_full_user_info_for_uuid(&uuid).await?;
        if info.is_none() {
            // Сброс куки с идентификатором
            id.forget();

            return Ok(None);
        }else{
            return Ok(info);
        }
    }else{
        return Ok(None);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn index(handlebars: web::Data<Handlebars<'_>>, 
               id: Identity,
               db: web::Data<Database>) -> Result<web::HttpResponse, AppError> {
    // Проверка идентификатора пользователя
    // TODO: приходится делать это здесь, а не в middleware, так как
    // есть проблемы с асинхронным запросом к базе в middleware 
    let full_info = match get_full_user_info_for_identity(&id, &db).await? {
        Some(full_info) => full_info,
        None => {
            // Возвращаем код 302 и Location в заголовках для перехода
            return Ok(web::HttpResponse::Found()
                        .header(actix_web::http::header::LOCATION, constants::LOGIN_PATH)
                        .finish())
        }
    };

    let template_data = serde_json::json!({
        "uuid": full_info.user_uuid,
        "facebook_uid": full_info.facebook_uid,
        "google_uid": full_info.google_uid,
    });

    // Рендерим шаблон
    let body = handlebars.render(constants::INDEX_TEMPLATE, &template_data)?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn login_page(handlebars: web::Data<Handlebars<'_>>,
                    id: Identity,
                    db: web::Data<Database>) -> Result<web::HttpResponse, AppError> {
    // Проверка идентификатора пользователя
    // TODO: приходится делать это здесь, а не в middleware, так как
    // есть проблемы с асинхронным запросом к базе в middleware 
    if get_uuid_from_ident_with_db_check(&id, &db).await?.is_some() {
        // Возвращаем код 302 и Location в заголовках для перехода
        return Ok(web::HttpResponse::Found()
                    .header(actix_web::http::header::LOCATION, constants::INDEX_PATH)
                    .finish())
    }

    let body = handlebars.render(constants::LOGIN_TEMPLATE, &serde_json::json!({}))?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn logout(id: Identity) -> Result<web::HttpResponse, AppError> {
    id.forget();

    // Возвращаем код 302 и Location в заголовках для перехода
    return Ok(web::HttpResponse::Found()
                .header(actix_web::http::header::LOCATION, constants::LOGIN_PATH)
                .finish())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Создаем менеджер шаблонов и регистрируем туда нужные
fn create_templates<'a>() -> Handlebars<'a> {
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
    
    handlebars
}

/// Функция непосредственного конфигурирования приложения
/// Для каждого потока исполнения будет создано свое приложение
fn configure_new_app(config: &mut web::ServiceConfig) {
    config
        .service(web::resource(constants::INDEX_PATH)
                    .route(web::route()
                            .guard(guard::Get())
                            .to(index)))
        .service(web::resource(constants::LOGIN_PATH)
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
        .service(Files::new("static/js", "static/js"));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Инициализируем менеджер логирования
    env_logger::init();

    // Получаем параметры Facebook + Google
    let facebook_env_params = web::Data::new(FacebookEnvParams::get_from_env());
    let google_env_params = web::Data::new(GoogleEnvParams::get_from_env());

    // Создаем подключение к нашей базе
    let sqlite_conn = web::Data::new(Database::open().await);

    // Создаем шареную ссылку на обработчик шаблонов
    // Пример работы с шаблонами
    // https://github.com/actix/examples/tree/master/template_engines/handlebars
    let handlebars = web::Data::new(create_templates());

    // Ключ для шифрования кук, генерируется каждый раз при запуске сервера
    let private_key = rand::thread_rng().gen::<[u8; 32]>();

    // Создаем общего http клиента для разных запросов
    let http_client = web::Data::new(reqwest::Client::new());

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

            // TODO: Session middleware

            // Приложение создается для каждого потока свое собственное
            // Порядок выполнения Middleware обратный, снизу вверх
            App::new()
                .wrap(identity_middleware)
                .wrap(create_error_middleware())
                .wrap(actix_web::middleware::Logger::default())
                .app_data(sqlite_conn.clone())
                .app_data(handlebars.clone())
                .app_data(facebook_env_params.clone())
                .app_data(google_env_params.clone())
                .app_data(http_client.clone())
                .configure(configure_new_app)
        }) 
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

/*fn main(){
    let mut runtime = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async_main()).unwrap();
}*/