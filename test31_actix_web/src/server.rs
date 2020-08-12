
use actix_web::{
    web, 
};
use crate::{
    app_state::{
        AppState
    },
    services::{
        configure_api_service,
        configure_http_service
    }
};


// Документация
// https://actix.rs/docs/

// this function could be located in different module
pub fn configure_server(cfg: &mut web::ServiceConfig) {
    // Создаем данные нашего состояния и устанавливаем
    let app_data = AppState::new("AppData");
    cfg.data(app_data); 

    // Настраиваем пространство от корня
    let http_scope = web::scope("")
        .configure(configure_http_service);

    // Настраиваем пространство API
    let api_scope = web::scope("/api")
        .configure(configure_api_service);

    cfg
        .service(api_scope)
        .service(http_scope);
}