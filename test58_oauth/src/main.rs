mod error;
mod facebook_env_params;
mod app_middlewares;
mod constants;
mod responses;

use std::{
    env::{
        self
    }
};
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
use sqlx::{
    prelude::{
        *
    },
    sqlite::{
        SqlitePool
    }
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
    },
    responses::{
        DataOrErrorResponse,
        FacebookErrorResponse,
        FacebookTokenResponse,
        FacebookUserInfoResponse
    }
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn index(handlebars: web::Data<Handlebars<'_>>) -> Result<web::HttpResponse, AppError> {
    let body = handlebars.render(constants::INDEX_TEMPLATE, &serde_json::json!({}))?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn login_page(handlebars: web::Data<Handlebars<'_>>) -> Result<web::HttpResponse, AppError> {
    let body = handlebars.render(constants::LOGIN_TEMPLATE, &serde_json::json!({}))?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Данный метод вызывается при нажатии на кнопку логина в Facebook
async fn login_with_facebook(req: actix_web::HttpRequest, fb_params: web::Data<FacebookEnvParams>) -> Result<web::HttpResponse, AppError> {
    debug!("Request object: {:?}", req);

    // Адрес нашего сайта + адрес коллбека
    /*let callback_site_address = {
        let site_addr = req
            .headers()
            .get(actix_web::http::header::ORIGIN)
            .and_then(|val|{
                val.to_str().ok()
            })
            .ok_or_else(||{
                AppError::ActixError(actix_web::error::ErrorBadRequest("Origin header get failed"))
            })?;
        format!("{}/facebook/auth_callback", site_addr)
    };*/
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
                                db: web::Data<SqlitePool>) -> Result<web::HttpResponse, AppError> {

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
    let user_info = http_client
        .get("https://graph.facebook.com/me")
        .query(&[
            ("access_token", response.access_token.as_str())
        ])
        .send()
        .await?
        .json::<DataOrErrorResponse<FacebookUserInfoResponse, FacebookErrorResponse>>()
        .await?
        .into_result()?;

    debug!("Facebook user info response: {:?}", user_info);

    // Получили айдишник пользователя на FB, делаем запрос к базе данных, чтобы проверить наличие нашего пользователя
    #[derive(Debug)]
    struct UserInfo {
        user_uuid: String
    }
    let db_res = sqlx::query_as!(UserInfo,
                                r#"   
                                    SELECT app_users.user_uuid 
                                    FROM app_users 
                                    INNER JOIN facebook_users 
                                            ON facebook_users.app_user_id = app_users.id
                                    WHERE facebook_users.facebook_uid = ?
                              "#, user_info.id)
        .fetch_optional(db.as_ref())
        .await?;

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

            // Стартуем транзакцию, если будет ошибка, то вызовется rollback автоматически в drop
            // если все хорошо, то руками вызываем commit
            let transaction = db.begin().await?;

            // TODO: ???
            // Если таблица иммет поле INTEGER PRIMARY KEY тогда last_insert_rowid - это алиас
            // Но вроде бы наиболее надежный способ - это сделать подзапрос
            let new_row_id = sqlx::query!(r#"
                            INSERT INTO app_users(user_uuid)
                                VALUES (?);
                            INSERT INTO facebook_users(facebook_uid, app_user_id)
                                VALUES (?, (SELECT id FROM app_users WHERE user_uuid = ?));
                        "#, uuid, user_info.id, uuid)
                .execute(db.as_ref())
                .await?
                .last_insert_rowid();

            transaction.commit().await?;

            debug!("New facebook user included: row_id = {}", new_row_id);

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
                                        .guard(guard::Get())
                                        .to(facebook_auth_callback))))
        .service(Files::new("static/css", "static/css"))
        .service(Files::new("static/js", "static/js"));
}

async fn create_db_connection() -> SqlitePool {
    let sqlite_conn = SqlitePool::connect(&env::var("DATABASE_URL")
                                             .expect("DATABASE_URL env variable is missing"))
        .await
        .expect("Database connection failed");

    // Включаем все миграции базы данных сразу в наш бинарник, выполняем при старте
    sqlx::migrate!("./migrations")
        .run(&sqlite_conn)
        .await
        .expect("database migration failed");

    sqlite_conn
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Инициализируем менеджер логирования
    env_logger::init();

    // Получаем параметры Facebook
    let facebook_env_params = web::Data::new(FacebookEnvParams::get_from_env());

    // Создаем подключение к нашей базе
    let sqlite_conn = web::Data::new(create_db_connection().await);

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
            App::new()
                .wrap(create_error_middleware())
                .wrap(identity_middleware)
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