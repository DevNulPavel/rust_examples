
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
    instrument,
    debug_span, 
    // debug,
};
use crate::{
    error::{
        AppError
    },
    database::{
        UserInfo
    },
    constants::{
        self
    }
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[instrument(skip(handlebars), fields(user_id = %full_info.user_uuid))]
pub async fn index(handlebars: web::Data<Handlebars<'_>>, 
                   full_info: UserInfo) -> Result<web::HttpResponse, AppError> {
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

#[instrument(skip(handlebars))]
pub async fn login_page(handlebars: web::Data<Handlebars<'_>>) -> Result<web::HttpResponse, AppError> {
    let body = handlebars.render(constants::LOGIN_TEMPLATE, &serde_json::json!({}))?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[instrument(skip(id))]
pub async fn logout(id: Identity) -> Result<web::HttpResponse, AppError> {
    id.forget();

    // Возвращаем код 302 и Location в заголовках для перехода
    return Ok(web::HttpResponse::Found()
                .header(actix_web::http::header::LOCATION, constants::LOGIN_PATH)
                .finish())
}