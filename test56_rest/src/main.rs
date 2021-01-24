mod ffi;
mod rest;
#[cfg(test)] mod tests;

use std::{
    io::{
        self
    }
};
use log::{
    info,
    //debug,
    //error
};
use actix_web::{
    middleware::{
        self
    },
    dev::{
        self
    },
    web::{
        self
    },
    guard::{
        self
    },
    HttpServer,
    App
};
use self::{
    rest::{
        upload_image_multipart,
        upload_image_json
    }
};

////////////////////////////////////////////////////////////////////////////////

fn build_image_service() -> impl dev::HttpServiceFactory {
    let upload_json_route = web::route()
        .guard(guard::Post())
        .guard(guard::Header("Content-Type", "application/json"))
        .to(upload_image_json);

    let upload_mulipart_route = web::route()
        .guard(guard::Post())
        .guard(guard::fn_guard(|req|{
            if let Some(val) = req.headers.get("Content-Type") {
                if let Ok(val_str) = val.to_str(){
                    return val_str.starts_with("multipart/form-data");
                }
            }
            false
        }))
        .to(upload_image_multipart);

    let image_service = web::resource("/upload_image")
        .route(upload_json_route)
        .route(upload_mulipart_route);
    
    image_service
}

////////////////////////////////////////////////////////////////////////////////

async fn start_server(addr: &str) -> io::Result<dev::Server>{
    // Клиент для запросов
    let shared_client = reqwest::Client::new();

    // Важно! На каждый поток у нас создается свое приложение со своими данными
    let app_builder = move ||{
        let image_service = build_image_service();
        App::new()
            .wrap(middleware::Logger::default())
            .default_service(web::route().to(|| { web::HttpResponse::NotFound() }))
            .service(image_service)
            .data(shared_client.clone())
            // .app_data() // Можно получить у запроса
            // .data(data) // Можно прокидывать как параметр у обработчика
    };

    // Запускаем сервер
    let server = HttpServer::new(app_builder)
        .bind(addr)?
        .keep_alive(75_usize) // 75 секунд
        .run();

    Ok(server)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Debug) // Для тестовых целей ставим всегда DEBUG
        .try_init()
        .expect("Logger init failed");

    let server = start_server("0.0.0.0:8080").await?;
    info!("Server started");
    
    tokio::signal::ctrl_c().await.expect("Signal wait failed");
    info!("Stop signal received");

    server.stop(true).await;
    info!("Gracefull stop finished");

    Ok(())
}