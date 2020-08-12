
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

    // Настраиваем пространство API
    configure_api_service(cfg);

    // Настраиваем пространство от корня
    configure_http_service(cfg);
    
}