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
use quick_error::{
    ResultExt
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
        GoogleErrorResponse,
        GoogleTokenResponse,
        GoogleUserInfoResponse
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
                               google_params: web::Data<GoogleEnvParams>,
                               fb_params: web::Data<GoogleEnvParams>) -> Result<web::HttpResponse, AppError> {
    debug!("Request object: {:?}", req);

    // Адрес нашего сайта + адрес коллбека
    let callback_site_address = get_callback_address(&req);
    
    // Создаем урл, на который надо будет идти для логина
    // https://developers.google.com/identity/protocols/oauth2/web-server#httprest
    let mut auth_url = google_params.auth_uri.clone();
    auth_url
        .query_pairs_mut()
        .append_pair("client_id", &fb_params.client_id)
        .append_pair("redirect_uri", &callback_site_address)
        .append_pair("response_type", "code")
        .append_pair("access_type", "online")
        .append_pair("include_granted_scopes", "true")
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
    code: String
}
pub async fn google_auth_callback(req: actix_web::HttpRequest,
                                  query_params: web::Query<GoogleAuthParams>, 
                                  identity: Identity,
                                  google_params: web::Data<GoogleEnvParams>,
                                  http_client: web::Data<reqwest::Client>,
                                  db: web::Data<Database>) -> Result<web::HttpResponse, AppError> {

    debug!("Request object: {:?}", req);
    debug!("Google auth callback query params: {:?}", query_params);

    let callback_site_address = get_callback_address(&req);

    // Выполняем запрос для получения токена на основании кода у редиректа
    let response = http_client
        .post(google_params.token_uri.as_ref())
        .form(&serde_json::json!({
            "client_id": google_params.client_id.as_str(),
            "client_secret": google_params.client_secret.as_str(),
            "redirect_uri": callback_site_address.as_str(),   // TODO: Для чего он нужен?
            "code": query_params.code.as_str(),
            "grant_type": "authorization_code"
        }))
        .send()
        .await
        // .context("Google token reqwest send error")? // Может выдать секреты наружу
        // .error_for_status()
        .context("Google token reqwest status error")?
        .json::<DataOrErrorResponse<GoogleTokenResponse, GoogleErrorResponse>>()
        .await
        .context("Google token reqwest parse error")?
        .into_result()?;

    debug!("Google token request response: {:?}", response);

    // Выполняем запрос информации о пользователе
    let user_info_data = http_client
        .get("https://www.googleapis.com/oauth2/v1/userinfo")
        .bearer_auth(&response.access_token)
        .send()
        .await
        // .context("Google token reqwest send error")? // Может выдать секреты наружу
        // .error_for_status()
        .context("Google user data reqwest status error")?
        .json::<DataOrErrorResponse<GoogleUserInfoResponse, GoogleErrorResponse>>()
        .await
        .context("Google user data reqwest response parse error")?
        .into_result()?;

    debug!("Google user info: {:?}", user_info_data);

    // Получили айдишник пользователя на FB, делаем запрос к базе данных, чтобы проверить наличие нашего пользователя
    let db_res = db.try_to_find_user_uuid_with_google_id(&user_info_data.id).await?;

    debug!("Google database search result: {:?}", db_res);
    
    match db_res {
        Some(user_uuid) => {
            debug!("Our user exists in database with UUID: {:?}", user_uuid);

            // Сохраняем идентификатор в куках
            identity.remember(user_uuid);
        },
        None => {
            // Выполняем генерацию UUID и запись в базу
            let uuid = format!("island_uuid_{}", uuid::Uuid::new_v4());
            
            // Сохраняем в базу идентификатор нашего пользователя
            db.insert_uuid_for_google_user(&uuid, &user_info_data.id).await?;

            // Сохраняем идентификатор в куках
            identity.remember(uuid);
        }
    }


    // Возвращаем код 302 и Location в заголовках для перехода
    Ok(web::HttpResponse::Found()
        .header(actix_web::http::header::LOCATION, constants::INDEX_PATH)
        .finish())
}