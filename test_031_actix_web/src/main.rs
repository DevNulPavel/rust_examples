mod app_state;
mod server;
mod services;
mod middlewares;

use listenfd::{
    ListenFd
};
use actix_web::{
    web,
    middleware,
    App, 
    HttpServer
};
use actix_session::{
    CookieSession
};
// use openssl::ssl::{
//     SslAcceptor, 
//     SslFiletype, 
//     SslMethod
// };
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
        // Создаем непосредственно приложение, конфигурируя его в другом месте
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .wrap(middlewares::check_login::CheckLogin::default())
            .wrap(CookieSession::signed(&[0; 32]).secure(false))
            .default_service(web::route().to(|| { 
                web::HttpResponse::MethodNotAllowed() 
            }))
            .configure(configure_server)
    };

    /*// Вариант создания самоподписанного ключа:
    // openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'
    // Вариант с ручной подписью:
    // openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -sha256 -subj "/C=CN/ST=Fujian/L=Xiamen/O=TVlinux/OU=Org/CN=muro.lxd"
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())
        .unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem")
        .unwrap();*/


    // Создаем слушателя, чтобы просто переподключаться к открытому сокету при быстром рестарте
    let server = match ListenFd::from_env().take_tcp_listener(0)?{
        Some(listener) => {
            // Создаем сервер с уже имеющимся листнером
            HttpServer::new(build_web_application)
                .listen(listener)?
        },
        None => {
            /*HttpServer::new(app_lambda)
                .bind("127.0.0.1:8080")?
                .bind_openssl("127.0.0.1:8088", builder)?*/
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