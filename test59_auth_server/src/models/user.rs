use std::{
    borrow::{
        Borrow
    }, ops::{
        Deref
    }, 
    sync::{
        Arc
    }
};
use futures::{
    future::{
        BoxFuture
    }
};
use actix_web::{
    dev::{
        Payload
    },
    web::{
        self
    },
    HttpRequest,
    FromRequest
};
use actix_web_httpauth::{
    extractors::{
        bearer::{
            BearerAuth
        }
    }
};
use uuid::{
    Uuid
};
use chrono::{
    NaiveDateTime
};
use serde::{
    Serialize,
    Deserialize
};
use validator::{
    Validate
};
use tracing::{
    instrument
};
use sqlx::{
    PgPool
};
use crate::{
    error::{
        AppError
    },
    crypto::{
        verify_password,
        TokenService
    }
};

/////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct CreateUserConfig {
    pub user_login: String,
    pub email: String,
    pub password_hash: String,
    pub password_salt: String
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct UpdateUserConfig {
    pub full_name: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>
}

/////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub struct UserData {
    pub id: Uuid,
    pub user_login: String,
    pub email: String,
    pub password_hash: String,
    pub password_salt: String,
    pub full_name: Option<String>,
    pub bio: Option<String>,
    pub user_image: Option<String>,
    pub create_at: NaiveDateTime,
    pub updated_at: NaiveDateTime
}

/////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct User{
    db: Arc<PgPool>,
    data: UserData
}

impl User {
    #[instrument]
    pub async fn create_new(db: Arc<PgPool>, info: CreateUserConfig) -> Result<User, AppError>{
        // TODO: Где валидировать?
        // info.validate()?;

        let data = sqlx::query_as!(UserData,
                r#"
                    INSERT INTO users(user_login, email, password_hash, password_salt)
                    VALUES ($1, $2, $3, $4) 
                    RETURNING *
                "#, info.user_login, info.email, info.password_hash, info.password_salt)
            .fetch_one(db.as_ref())
            .await
            .map_err(AppError::from)?;
        Ok(User{
            db,
            data
        })
    }

    #[instrument(fields(id = %id.borrow()))]
    pub async fn find_by_uuid<ID: Borrow<Uuid>>(db: Arc<PgPool>, id: ID) -> Result<Option<User>, AppError> {
        let user_opt = sqlx::query_as!(UserData,
                r#"
                    SELECT *
                    FROM users
                    WHERE id = $1
                "#, id.borrow())
            .fetch_optional(db.as_ref())
            .await
            .map_err(AppError::from)?
            .map(|data| {
                User{
                    data,
                    db
                }
            });
        Ok(user_opt)
    }

    #[instrument]
    pub async fn find_by_login(db: Arc<PgPool>, login: &str) -> Result<Option<User>, AppError> {
        let user_opt = sqlx::query_as!(UserData,
                r#"
                    SELECT *
                    FROM users
                    WHERE user_login = $1
                "#, login)
            .fetch_optional(db.as_ref())
            .await
            .map_err(AppError::from)?
            .map(|data| {
                User{
                    data,
                    db
                }
            });
        Ok(user_opt)
    }

    #[instrument]
    pub async fn update_profile_info(&mut self, info: UpdateUserConfig) -> Result<(), AppError>{
        let new_data = sqlx::query_as!(UserData,
                r#"
                    UPDATE users
                    SET full_name = $1, bio = $2, user_image = $3
                    WHERE id = $4
                    RETURNING *
                "#, info.full_name, info.bio, info.image, self.data.id)
            .fetch_one(self.db.as_ref())
            .await
            .map_err(AppError::from)?;
        self.data = new_data;
        Ok(())
    }

    pub fn get_data(&self) -> &UserData{
        &self.deref()
    }

    pub async fn verify_password(&self, password: &str) -> Result<bool, AppError>{
        verify_password(password.as_bytes().to_owned(), self.data.password_hash.clone()).await
    }
}

impl std::ops::Deref for User{
    type Target = UserData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl FromRequest for User{
    type Config = ();
    type Error = AppError;
    type Future = BoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future{
        let auth = BearerAuth::from_request(req, payload).into_inner();
        let db = web::Data::<PgPool>::from_request(req, payload).into_inner();
        let token_service = web::Data::<TokenService>::from_request(req, payload).into_inner();

        match (auth, db, token_service) {
            (Ok(auth), Ok(db), Ok(token_service)) => {
                Box::pin(async move{
                    // Декодируем токен
                    let token_data = token_service
                        .decode_jwt_token(auth.token().to_string())
                        .await?;

                    // Проверяем, что токен еще живой
                    if !token_data.is_not_expired() {
                        return Err(AppError::UnautorisedError("Token is expired"));
                    }

                    let found_user = User::find_by_uuid(db.into_inner(), token_data.uuid)
                        .await?
                        .ok_or_else(||{
                            AppError::UnautorisedError("User is missing in database")
                        })?;

                    Ok(found_user)
                })
            },
            _ => {
                Box::pin(async{
                    Err(AppError::UnautorisedError("Auth error"))
                })
            }
        }
    }
}