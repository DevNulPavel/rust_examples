use actix_web::{
    web::{
        self
    }
};
use tracing::{
    debug
};
use actix_identity::{
    Identity
};
use serde::{
    Deserialize
};
use lazy_static::{
    lazy_static
};
use quick_error::{
    ResultExt
};
use tracing::{
    instrument
};
use crate::{
    error::{
        AppError
    },
    env_app_params::{
        FacebookEnvParams
    },
    responses::{
        DataOrErrorResponse,
        FacebookErrorResponse,
        FacebookTokenResponse,
        FacebookUserInfoResponse
    },
    database::{
        Database
    },
    constants::{
        self
    }
};

fn get_callback_address(req: &actix_web::HttpRequest) -> String {
    let conn_info = req.connection_info();
    format!("{scheme}://{host}{api}{login}", 
                scheme = conn_info.scheme(),
                host = conn_info.host(),
                api = constants::FACEBOOK_SCOPE_PATH,
                login = constants::AUTH_CALLBACK_PATH)
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Данный метод вызывается при нажатии на кнопку логина в Facebook
#[instrument(fields(callback_site_address))]
pub async fn login_with_facebook(req: actix_web::HttpRequest, 
                                 fb_params: web::Data<FacebookEnvParams>) -> Result<web::HttpResponse, AppError> {
    debug!("Request object: {:?}", req);

    // Адрес нашего сайта + адрес коллбека
    let callback_site_address = get_callback_address(&req);

    tracing::Span::current()
        .record("callback_site_address", &tracing::field::display(&callback_site_address));

    // Создаем урл, на который надо будет идти для логина
    // https://www.facebook.com/dialog/oauth\
    //      ?client_id=578516362116657\
    //      &redirect_uri=http://localhost/facebook-auth\
    //      &response_type=code\
    //      &scope=email,user_birthday
    lazy_static! {
        // Мелкая оптимизация, чтобы бестолку не парсить базовый URL каждый раз
        static ref AUTH_URL: url::Url = url::Url::parse("https://www.facebook.com/dialog/oauth").unwrap();
    }
    let mut auth_url = AUTH_URL.clone();
    auth_url
        .query_pairs_mut()
        .append_pair("client_id", &fb_params.client_id)
        .append_pair("redirect_uri", &callback_site_address)
        .append_pair("response_type", "code")
        .append_pair("scope", "email")
        .finish();

    debug!("Facebook url value: {}", auth_url);

    // Возвращаем код 302 и Location в заголовках для перехода
    Ok(web::HttpResponse::Found()
        .header(actix_web::http::header::LOCATION, auth_url.as_str())
        .finish())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Данный метод является адресом-коллбеком который вызывается после логина на facebook
#[derive(Debug, Deserialize)]
pub struct FacebookAuthParams{
    code: String,
    // scope: String
}
#[instrument(skip(identity), fields(callback_site_address))]
pub async fn facebook_auth_callback(req: actix_web::HttpRequest,
                                    query_params: web::Query<FacebookAuthParams>, 
                                    identity: Identity,
                                    fb_params: web::Data<FacebookEnvParams>,
                                    http_client: web::Data<reqwest::Client>,
                                    db: web::Data<Database>) -> Result<web::HttpResponse, AppError> {
    debug!("Request object: {:?}", req);
    debug!("Facebook auth callback query params: {:?}", query_params);

    let callback_site_address = get_callback_address(&req);

    tracing::Span::current()
        .record("callback_site_address", &tracing::field::display(&callback_site_address));

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
        // .instrument(tracing::info_span!("facebook_token"))
        .await
        .context("Facebook token reqwest send error")?
        // .error_for_status() // Может выдать секреты наружу
        // .context("Facebook token reqwest status error")?
        .json::<DataOrErrorResponse<FacebookTokenResponse, FacebookErrorResponse>>()
        .await
        .context("Facebook token reqwest parse error")?
        .into_result()?;

    debug!("Facebook token request response: {:?}", response);

    // Теперь можем получить информацию о пользователе Facebook
    let fb_user_info = http_client
        .get("https://graph.facebook.com/me")
        .query(&[
            ("access_token", response.access_token.as_str())
        ])
        .send()
        .await
        .context("Facebook user data reqwest send error")?
        // .error_for_status() // Может выдать секреты наружу
        // .context("Facebook user data reqwest status error")?
        .json::<DataOrErrorResponse<FacebookUserInfoResponse, FacebookErrorResponse>>()
        .await
        .context("Facebook user data reponse parse error")?
        .into_result()?;

    debug!("Facebook user info response: {:?}", fb_user_info);

    // Получили айдишник пользователя на FB, делаем запрос к базе данных, чтобы проверить наличие нашего пользователя
    let db_res = db.try_to_find_user_uuid_with_fb_id(&fb_user_info.id).await?;

    debug!("Facebook database search result: {:?}", db_res);
    
    match db_res {
        Some(user_uuid) => {
            debug!("Our user exists in database with UUID: {:?}", user_uuid);

            // Сохраняем идентификатор в куках
            identity.remember(user_uuid);
        },
        None => {
            // Если мы залогинились, но у нас есть валидный пользователь в куках, джойним к нему GoogleId
            let uuid = match identity.identity() {
                Some(uuid) if db.does_user_uuid_exist(&uuid).await? => {
                    debug!(uuid = %uuid, "User with identity exists");

                    // Добавляем в базу идентификатор нашего пользователя
                    db.append_facebook_user_for_uuid(&uuid, &fb_user_info.id).await?;

                    uuid
                },
                _ => {
                    // Сбрасываем если был раньше
                    identity.forget();
                    
                    // TODO: вынести в общую функцию
                    // Выполняем генерацию нового UUID
                    let uuid = format!("island_uuid_{}", uuid::Uuid::new_v4());

                    // Сохраняем в базу идентификатор нашего пользователя
                    db.insert_facebook_user_with_uuid(&uuid, &fb_user_info.id).await?;

                    uuid
                }
            };

            // Сохраняем идентификатор в куках
            identity.remember(uuid);
        }
    }

    // Возвращаем код 302 и Location в заголовках для перехода
    Ok(web::HttpResponse::Found()
        .header(actix_web::http::header::LOCATION, constants::INDEX_PATH)
        .finish())
}