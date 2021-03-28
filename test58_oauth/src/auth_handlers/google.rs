use actix_web::{
    web::{
        self
    }
};
use log::{
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
use crate::{
    error::{
        AppError
    },
    env_app_params::{
        GoogleEnvParams
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
                api = constants::GOOGLE_SCOPE_PATH,
                login = constants::AUTH_CALLBACK_PATH)
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Данный метод вызывается при нажатии на кнопку логина в Facebook
pub async fn login_with_google(req: actix_web::HttpRequest, 
                                 fb_params: web::Data<GoogleEnvParams>) -> Result<web::HttpResponse, AppError> {
    debug!("Request object: {:?}", req);

    // Адрес нашего сайта + адрес коллбека
    let callback_site_address = get_callback_address(&req);

    // Создаем урл, на который надо будет идти для логина
    lazy_static! {
        // Мелкая оптимизация, чтобы бестолку не парсить базовый URL каждый раз
        static ref AUTH_URL: url::Url = url::Url::parse("https://accounts.google.com/o/oauth2/auth")
            .unwrap();
    }
    let mut auth_url = AUTH_URL.clone();
    auth_url
        .query_pairs_mut()
        .append_pair("client_id", &fb_params.client_id)
        .append_pair("redirect_uri", &callback_site_address)
        .append_pair("response_type", "code")
        .append_pair("scope", "https://www.googleapis.com/auth/userinfo.email")
        .finish();

    debug!("Google url value: {}", auth_url);

    // Возвращаем код 302 и Location в заголовках для перехода
    Ok(web::HttpResponse::Found()
        .header(actix_web::http::header::LOCATION, auth_url.as_str())
        .finish())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Данный метод является адресом-коллбеком который вызывается после логина на facebook
#[derive(Debug, Deserialize)]
pub struct GoogleAuthParams{
    code: String,
    //scope: String
}
pub async fn google_auth_callback(req: actix_web::HttpRequest,
                                  query_params: web::Query<GoogleAuthParams>, 
                                  identity: Identity,
                                  google_params: web::Data<GoogleEnvParams>,
                                  http_client: web::Data<reqwest::Client>,
                                  db: web::Data<Database>) -> Result<web::HttpResponse, AppError> {

    debug!("Request object: {:?}", req);
    debug!("Google auth callback query params: {:?}", query_params);

    // Возвращаем код 302 и Location в заголовках для перехода
    Ok(web::HttpResponse::Found()
        .header(actix_web::http::header::LOCATION, constants::INDEX_PATH)
        .finish())
}