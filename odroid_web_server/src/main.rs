mod server;
mod services;
mod middlewares;
mod constants;

use listenfd::{
    ListenFd
};
use actix_web::{
    web,
    middleware,
    App, 
    HttpServer
};
use actix_identity::{
    CookieIdentityPolicy, 
    IdentityService
};
use rand::{
    Rng
};
use crate::{
    server::{
        configure_server
    }
};

#[actix_rt::main]
async fn main() -> std::io::Result<()>{
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    let build_web_application = ||{        
        let identity_middleware = {
            let private_key = rand::thread_rng().gen::<[u8; 32]>();
            let policy = CookieIdentityPolicy::new(&private_key)
                .name("auth-logic")
                .max_age(60 * 60 * 24 * 14) // 14 дней максимум
                .secure(false);
            IdentityService::new(policy)
        };

        App::new()
            .wrap(identity_middleware)
            //.wrap(middlewares::check_login::CheckLogin::default())
            .wrap(middleware::Logger::default())
            .default_service(web::route().to(|| { 
                web::HttpResponse::MethodNotAllowed() 
            }))
            .configure(configure_server)
    };

    // Создаем слушателя, чтобы просто переподключаться к открытому сокету при быстром рестарте
    let server = match ListenFd::from_env().take_tcp_listener(0)?{
        Some(listener) => {
            // Создаем сервер с уже имеющимся листнером
            HttpServer::new(build_web_application)
                .listen(listener)?
        },
        None => {
            // Создаем новый сервер
            HttpServer::new(build_web_application)
                .bind("127.0.0.1:8080")?
        }
    };

    // Запускаем сервер
    server
        .keep_alive(75_usize) // 75 секунд
        .workers(1_usize) // Можно задать конкретное количество потоков
        .run()
        .await
}