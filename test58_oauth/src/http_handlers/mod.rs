
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
pub async fn login_page(handlebars: web::Data<Handlebars<'_>>) -> Result<web::HttpResponse, AppError> {
    let span = debug_span!("login_page_span", "user" = tracing::field::Empty);
    let _enter_guard = span.enter();

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