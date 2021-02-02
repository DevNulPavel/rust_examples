mod validate_params;
mod shared_data;
mod auth;
mod payment_info;
mod process_pay;
mod static_forward;

use std::{
    io::{
        self
    }
};
use log::{
    info,
    debug,
    error
};
use url::{
    Url
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
    // guard::{
    //     self
    // },
    client::{
        Client
    },
    HttpServer,
    App
};
use self::{
    shared_data::{
        SharedAppData
    },
    auth::{
        auth
    },
    payment_info::{
        get_payment_info
    },
    process_pay::{
        process_pay
    },
    static_forward::{
        static_forward
    }
};

////////////////////////////////////////////////////////////////////////////////

async fn start_server(addr: &str) -> io::Result<dev::Server>{
    let static_redirect_url = Url::parse("http://127.0.0.1:8080").unwrap();

    let shared_data = web::Data::new(SharedAppData{
        secret_key: String::from("TEST")
    });

    // Важно! На каждый поток у нас создается свое приложение со своими данными
    let app_builder = move ||{
        let auth_service = web::resource("/auth")
            .route(web::get().to(auth));
        
        let get_payment_info_service = web::resource("/get_payment_info")
            .route(web::post().to(get_payment_info));
        
        let process_pay_service = web::resource("/process_pay")
            .route(web::post().to(process_pay));

        let static_files_service = web::resource("/azerion")
            .route(web::route().to(static_forward))
            .data(Client::new()) // Можно прокидывать как параметр у обработчика
            .data(static_redirect_url.clone()); // Можно прокидывать как параметр у обработчика
        
        App::new()
            .wrap(middleware::Logger::default())
            .default_service(web::route().to(|| { web::HttpResponse::NotFound() }))
            .service(auth_service)
            .service(get_payment_info_service)
            .service(process_pay_service)
            .service(static_files_service)
            .data(shared_data.clone()) // Можно прокидывать как параметр у обработчика
            
            // .app_data(data) // Как параметр у запроса
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

    let server = start_server("0.0.0.0:8888").await?;
    info!("Server started");
    
    tokio::signal::ctrl_c().await.expect("Signal wait failed");
    info!("Stop signal received");

    server.stop(true).await;
    info!("Gracefull stop finished");

    Ok(())
}