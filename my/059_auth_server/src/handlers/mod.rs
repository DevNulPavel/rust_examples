use actix_web::{
    web::{
        self
    },
    guard::{
        self
    },
    HttpResponse
};
use actix_web_httpauth::{
    extractors::{
        basic::{
            BasicAuth
        }
    }
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
use rand::{
    distributions::{
        Alphanumeric
    },
    Rng,
    thread_rng
};
use crate::{
    error::{
        AppError
    },
    models::{
        user::{
            User,
            CreateUserConfig,
            UpdateUserConfig
        }
    },
    crypto::{
        TokenService,
        hash_password_with_salt
    }
};

//////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserReqData {
    #[validate(length(min = 3))] // TODO: валидация, чтобы не было пробелов
    pub user_login: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 3))]
    pub password: String
}

#[instrument]
async fn signup(req_params: web::Json<CreateUserReqData>, 
                db: web::Data<PgPool>) -> Result<HttpResponse, AppError> {

    let data: CreateUserReqData = req_params.into_inner();

    // TODO: Middleware для валидации
    data
        .validate()
        .context("New user data error")?;
    
    // Рандомная соль
    let random_salt: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    // Хешируем
    let password_hash = hash_password_with_salt(data.password.into_bytes(), random_salt.as_bytes().to_owned())
        .await?;

    // Запись в базу пользователя
    let config = CreateUserConfig{
        email: data.email,
        user_login: data.user_login,
        password_hash: password_hash,
        password_salt: random_salt
    };
    let new_user = User::create_new(db.into_inner(), config).await?;

    // Отдать в виде json
    Ok(HttpResponse::Ok().json(new_user.get_data()))
}

//////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateProfileReqData {
    pub full_name: Option<String>,
    pub bio: Option<String>,
    #[validate(url)]
    pub image_url: Option<String>
}

#[instrument]
async fn update_user_data(req_params: web::Json<UpdateProfileReqData>, 
                          mut user: User) -> Result<HttpResponse, AppError> {
    req_params
        .validate()
        .context("User update data errors")?;

    let data = req_params.into_inner();

    let config = UpdateUserConfig{
        bio: data.bio,
        full_name: data.full_name,
        image_url: data.image_url
    };
    user.update_profile_info(config).await?;

    Ok(HttpResponse::Ok().json(user.get_data()))
}

//////////////////////////////////////////////////////////////////////////////////////////

#[instrument]
async fn get_user_data(user: User) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(user.get_data()))
}

//////////////////////////////////////////////////////////////////////////////////////////

/// Обработчик аутентификации, принимающий логин и пароль, затем выдающий токен
#[instrument]
async fn auth(basic_auth: BasicAuth, 
              token_service: web::Data<TokenService>, 
              db: web::Data<PgPool>) -> Result<HttpResponse, AppError> {
    // Выдергиваем логин из запроса
    let login = basic_auth
        .user_id();
    
    // Выдергиваем пароль из запроса
    let password = basic_auth
        .password()
        .ok_or_else(|| AppError::UnautorisedError("Password is required for auth"))?;    

    // Находим пользователя
    let user = User::find_by_login(db.into_inner(), login)
        .await?
        .ok_or_else(|| AppError::UnautorisedError("User with login did not found"))?;

    // Сравшиваем пароль и хэш пользователя
    let valid_password = user
        .verify_password(password)
        .await?;

    // Если невалидный пароль, значит выходим
    if !valid_password{
        return Err(AppError::UnautorisedError("Wrong password"));
    }

    // Иначе, выдаем токен
    let token = token_service
        .generate_jwt_token(user.id)
        .await?;

    Ok(HttpResponse::Ok().json(token))
}


//////////////////////////////////////////////////////////////////////////////////////////

/// Функция непосредственного конфигурирования приложения
/// Для каждого потока исполнения будет создано свое приложение
pub fn configure_routes(config: &mut web::ServiceConfig) {
    config
        .service(web::resource("/signup")
                    .route(web::route()
                            .guard(guard::Post())
                            .to(signup)))
        .service(web::resource("/auth")
                    .route(web::route()
                    .guard(guard::Post())
                    .to(auth)))                            
        .service(web::resource("/user")
                    .route(web::route()
                            .guard(guard::Patch())
                            .to(update_user_data))
                    .route(web::route()
                            .guard(guard::Get())
                            .to(get_user_data)));
}
