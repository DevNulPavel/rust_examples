
use actix_web::{
    web, 
};
use crate::{
    services::{
        http_service
    }
};

pub fn configure_server(cfg: &mut web::ServiceConfig) {
    // Настраиваем пространство от корня
    let http_scope = web::scope("")
        .configure(http_service::configure_http_service);

    cfg
        .service(http_scope);
}