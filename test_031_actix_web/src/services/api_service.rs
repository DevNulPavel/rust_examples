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
        .content_type("application/text")
        .body("Get status response")
}

pub fn configure_api_service(cfg: &mut ServiceConfig){
    cfg.route("/get_status", web::get().to(get_status));
}