
use actix_web::{
    web, 
};
use actix_files::{
    Files
};
use crate::{
    services::{
        http_service
    }
};

pub fn configure_server(cfg: &mut web::ServiceConfig) {
    // Отдача статики
    let static_css_files_service = Files::new("/static/css", "static/css/");
    let static_js_files_service = Files::new("/static/js", "static/js/");

    // Настраиваем пространство от корня
    let http_scope = web::scope("")
        .configure(http_service::configure_http_service);
    
    cfg
        .service(static_css_files_service)
        .service(static_js_files_service)
        .service(http_scope);
}