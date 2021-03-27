mod error;
mod facebook_env_params;
mod app_middlewares;
mod constants;
mod responses;
mod database;

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
use serde::{
    Deserialize
};
use crate::{
    error::{
        AppError
    },
    facebook_env_params::{
        FacebookEnvParams
    },
    app_middlewares::{
        create_error_middleware
    },
    responses::{
        DataOrErrorResponse,
        FacebookErrorResponse,
        FacebookTokenResponse,
        FacebookUserInfoResponse
    },
    database::{
        Database
    }
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn get_uuid_with_db_check(id: &Identity,
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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn index(handlebars: web::Data<Handlebars<'_>>, 
               id: Identity,
               db: web::Data<Database>) -> Result<web::HttpResponse, AppError> {
    // Проверка идентификатора пользователя
    // TODO: приходится делать это здесь, а не в middleware, так как
    // есть проблемы с асинхронным запросом к базе в middleware 
    let uuid = match get_uuid_with_db_check(&id, &db).await? {
        Some(uuid) => uuid,
        None => {
            // Возвращаем код 302 и Location в заголовках для перехода
            return Ok(web::HttpResponse::Found()
                        .header(actix_web::http::header::LOCATION, constants::LOGIN_PATH)
                        .finish())
        }
    };

    let template_data = serde_json::json!({
        "uuid": uuid
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
    if get_uuid_with_db_check(&id, &db).await?.is_some() {
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

/// Данный метод вызывается при нажатии на кнопку логина в Facebook
async fn login_with_facebook(req: actix_web::HttpRequest, fb_params: web::Data<FacebookEnvParams>) -> Result<web::HttpResponse, AppError> {
    debug!("Request object: {:?}", req);

    // Адрес нашего сайта + адрес коллбека
    let callback_site_address = {
        let conn_info = req.connection_info();
        format!("{scheme}://{host}/facebook/auth_callback", 
                    scheme = conn_info.scheme(),
                    host = conn_info.host())
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
        .append_pair("scope", "email")
        .finish();

    debug!("Facebook url value: {}", redirect_url);

    // Возвращаем код 302 и Location в заголовках для перехода
    Ok(web::HttpResponse::Found()
        .header(actix_web::http::header::LOCATION, redirect_url.as_str())
        .finish())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Данный метод является адресом-коллбеком который вызывается после логина на facebook
#[derive(Debug, Deserialize)]
pub struct FacebookAuthParams{
    code: String
}
async fn facebook_auth_callback(req: actix_web::HttpRequest,
                                query_params: web::Query<FacebookAuthParams>, 
                                identity: Identity,
                                fb_params: web::Data<FacebookEnvParams>,
                                http_client: web::Data<reqwest::Client>,
                                db: web::Data<Database>) -> Result<web::HttpResponse, AppError> {

    let callback_site_address = {
        let conn_info = req.connection_info();
        format!("{scheme}://{host}/facebook/auth_callback", 
                    scheme = conn_info.scheme(),
                    host = conn_info.host())
    };

    debug!("Request object: {:?}", req);
    debug!("Facebook auth callback query params: {:?}", query_params);

    // Выполняем запрос для получения токена на основании кода у редиректа
    let response = http_client
        .get("https://graph.facebook.com/oauth/access_token")
        .query(&[
            ("client_id", fb_params.client_id.as_str()),
            ("redirect_uri", callback_site_address.as_str()),   // TODO: Для чего он нужен?
            ("client_secret", fb_params.client_secret.as_str()),
            ("code", query_params.code.as_str())
        ])
        .send()
        .await?
        .json::<DataOrErrorResponse<FacebookTokenResponse, FacebookErrorResponse>>()
        .await?
        .into_result()?;

    debug!("Facebook token request response: {:?}", response);

    // Теперь можем получить информацию о пользователе Facebook
    let fb_user_info = http_client
        .get("https://graph.facebook.com/me")
        .query(&[
            ("access_token", response.access_token.as_str())
        ])
        .send()
        .await?
        .json::<DataOrErrorResponse<FacebookUserInfoResponse, FacebookErrorResponse>>()
        .await?
        .into_result()?;

    debug!("Facebook user info response: {:?}", fb_user_info);

    // Получили айдишник пользователя на FB, делаем запрос к базе данных, чтобы проверить наличие нашего пользователя
    let db_res = db.try_to_find_user_with_fb_id(&fb_user_info.id).await?;

    debug!("Facebook database search result: {:?}", db_res);
    
    match db_res {
        Some(data) => {
            debug!("Our user exists in database with UUID: {:?}", data);

            // Сохраняем идентификатор в куках
            identity.remember(data.user_uuid);
        },
        None => {
            // Выполняем генерацию UUID и запись в базу
            let uuid = format!("island_uuid_{}", uuid::Uuid::new_v4());
            
            // Сохраняем в базу идентификатор нашего пользователя
            db.insert_uuid_for_facebook_user(&uuid, &fb_user_info.id).await?;

            // Сохраняем идентификатор в куках
            identity.remember(uuid);
        }
    }

    // Возвращаем код 302 и Location в заголовках для перехода
    Ok(web::HttpResponse::Found()
        .header(actix_web::http::header::LOCATION, constants::INDEX_PATH)
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
        .service(web::scope("/facebook")
                    .service(web::resource("/login")
                                .route(web::route()
                                        .guard(guard::Post())
                                        .to(login_with_facebook)))
                    .service(web::resource("/auth_callback")
                                .route(web::route()
                                        .guard(guard::Get())
                                        .to(facebook_auth_callback))))
        .service(Files::new("static/css", "static/css"))
        .service(Files::new("static/js", "static/js"));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Инициализируем менеджер логирования
    env_logger::init();

    // Получаем параметры Facebook
    let facebook_env_params = web::Data::new(FacebookEnvParams::get_from_env());

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