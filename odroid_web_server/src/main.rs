mod server;
mod services;
mod middlewares;
mod constants;
mod camera;
mod gpio;

use std::{
    fs::{
        File
    },
    io::{
        BufReader
    }
};
use log::{
    // debug,
    info,
    // error
};
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
/*use actix_web_middleware_redirect_https::{
    RedirectHTTPS
};*/
use rustls::{
    internal::{
        pemfile::{
            certs, 
            rsa_private_keys
        }
    },
    NoClientAuth, 
    ServerConfig
};
use rand::{
    Rng
};
use crate::{
    server::{
        configure_server
    }
};

/*fn build_rustls_config() -> ServerConfig{
    // https://github.com/actix/examples/tree/master/rustls
    //
    // Создание сертификата:
    //
    // brew install nss mkcert
    // mkcert 127.0.0.1
    // mv 127.0.0.1.pem cert.pem
    // mv 127.0.0.1-key.pem key.pem
    // openssl rsa -in key.pem -out key-rsa.pem

    // Создаем конфиг
    let mut config = ServerConfig::new(NoClientAuth::new());

    // Читаем файл сертификата
    let cert_chain = {
        let file = File::open("rustls_certificates/cert.pem")
            .expect("rustls_certificates/key.pem is not found");
        let mut reader = BufReader::new(file);
        certs(&mut reader)
            .expect("Rustls cectificate read failed")
    };

    // Читаем файл закрытого ключа
    let keys = {
        let file = File::open("rustls_certificates/key-rsa.pem")
            .expect("rustls_certificates/key.pem is not found");
        let mut reader = BufReader::new(file);
        let mut keys = rsa_private_keys(&mut reader)
            .expect("Rustls private key read failed");

        assert!(keys.len() > 0, "Rustls private keys is empty");                

        keys.remove(0)
    };

    config
        .set_single_cert(cert_chain, keys)
        .expect("Rustls set_single_cert failed");

    config
}*/

#[actix_rt::main]
async fn main() -> std::io::Result<()>{
    // https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info,odroid_web_server=trace");
    env_logger::init();

    info!("Application setup");

    let build_web_application = ||{
        // Middleware перекидывания всех запросов в https
        /*let redirect_to_https = RedirectHTTPS::with_replacements(&[
            (":8080".to_owned(), ":8443".to_owned())
        ]);*/

        // Настраиваем middleware идентификации пользователя   
        let identity_middleware = {
            let private_key = rand::thread_rng().gen::<[u8; 32]>();
            let policy = CookieIdentityPolicy::new(&private_key)
                .name("auth-logic")
                .max_age(60 * 60 * 24 * 30) // 30 дней максимум
                .secure(false);
            IdentityService::new(policy)
        };

        App::new()
            //.wrap(redirect_to_https)
            .wrap(identity_middleware)
            .wrap(middleware::Logger::default())
            .default_service(web::route().to(|| { 
                // web::HttpResponse::MethodNotAllowed()
                // web::HttpResponse::NotFound()
                web::HttpResponse::Found()
                    .header("location", "/")
                    .finish()
            }))
            .configure(configure_server)
            // TODO: default https://github.com/actix/examples/blob/master/basics/src/main.rs
            /*.default_service(
                // 404 for GET request
                web::resource("")
                    .route(web::get().to(p404))
                    // all requests that are not `GET`
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            )*/
            
    };

    // Конфигурация rustls
    //let rustls_config = build_rustls_config();

    // Создаем слушателя, чтобы просто переподключаться к открытому сокету при быстром рестарте
    let server = match ListenFd::from_env().take_tcp_listener(0)?{
        Some(listener) => {
            // Создаем сервер с уже имеющимся листнером
            HttpServer::new(build_web_application)
                .listen(listener)?
        },
        None => {
            // Создаем новый сервер
            // https://127.0.0.1:8443/login
            // http://127.0.0.1:8080/login
            HttpServer::new(build_web_application)
                //.bind_rustls("0.0.0.0:8443", rustls_config)?
                .bind("0.0.0.0:8080")?
        }
    };

    // Запускаем сервер
    server
        .keep_alive(75_usize) // 75 секунд
        .workers(1_usize) // Можно задать конкретное количество потоков
        .run()
        .await
}