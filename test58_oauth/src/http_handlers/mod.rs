
use actix_web::{
    web::{
        self
    }
};
use handlebars::{
    Handlebars
};
use actix_identity::{
    Identity
};
use tracing::{
    debug_span, 
    debug,
};
use crate::{
    error::{
        AppError
    },
    database::{
        Database,
        UserInfo
    },
    constants::{
        self
    },
    helpers::{
        get_full_user_info_for_identity,
        get_uuid_from_ident_with_db_check
    }
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl actix_web::FromRequest for UserInfo {
    type Config = ();
    type Error = actix_web::Error;
    type Future = futures::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_http::Payload) -> Self::Future{
        let ext = req.extensions();
        match ext.get::<UserInfo>() {
            Some(full_info) => {
                futures::future::ready(Ok(full_info.clone())) // TODO: Убрать клон?
            },
            None => {
                futures::future::ready(Err(actix_web::error::ErrorUnauthorized("User info is missing")))
            }
        }        
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// #[instrument]
pub async fn index(handlebars: web::Data<Handlebars<'_>>, 
                   full_info: UserInfo) -> Result<web::HttpResponse, AppError> {
    let span = debug_span!("index_page_span", "user" = tracing::field::Empty);
    let _enter_guard = span.enter();

    span.record("user", &tracing::field::debug(&full_info));

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

// #[instrument]
pub async fn login_page(handlebars: web::Data<Handlebars<'_>>,
                        id: Identity,
                        db: web::Data<Database>) -> Result<web::HttpResponse, AppError> {
    let span = debug_span!("login_page_span", "user" = tracing::field::Empty);
    let _enter_guard = span.enter();

    // Проверка идентификатора пользователя
    // TODO: приходится делать это здесь, а не в middleware, так как
    // есть проблемы с асинхронным запросом к базе в middleware 
    if get_uuid_from_ident_with_db_check(&id, &db).await?.is_some() {
        debug!("Redirect code from handler");
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

// #[instrument]
pub async fn logout(id: Identity) -> Result<web::HttpResponse, AppError> {
    id.forget();

    // Возвращаем код 302 и Location в заголовках для перехода
    return Ok(web::HttpResponse::Found()
                .header(actix_web::http::header::LOCATION, constants::LOGIN_PATH)
                .finish())
}