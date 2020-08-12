use actix_web::{
    web::{
        self,
        ServiceConfig
    },
    // guard::{
    //     self
    // },
    HttpResponse,
    Responder
};

async fn get_status() -> impl Responder{
    HttpResponse::Ok()
        .body("Get status response")
}

pub fn configure_api_service(cfg: &mut ServiceConfig){
    // Создаем сервис с префиксом /test
    // .guard(guard::Get()) // Обрабатываем только get запросы
    let api_scope = web::scope("/api")
        .route("/get_status", web::get().to(get_status));

    cfg.service(api_scope);
}