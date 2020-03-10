#![allow(unused_imports)]

use http::StatusCode;
use listenfd::ListenFd;
use actix_web::{
    get,
    post,
    web, 
    App, 
    HttpServer, 
    HttpRequest, 
    HttpResponse,
    Responder
};
use openssl::ssl::{
    SslAcceptor, 
    SslFiletype, 
    SslMethod
};

// Документация
// https://actix.rs/docs/

// This struct represents state
#[derive(Debug)]
struct AppState {
    app_name: String,
    counter: std::sync::Mutex<i32>
}

// http://127.0.0.1:8080/123/123/index.html
// Можно описывать путь с помощью макроса
#[get("/{id}/{name}/index.html")]
async fn index(info: web::Path<(u32, String)>) -> impl Responder {
    let text = format!("Hello {}! id:{}", info.1, info.0);
    //HttpResponse::with_body(StatusCode::from_u16(200).unwrap(), text)
    text
}

#[post("/test1")]
async fn hello1(info: web::Path<(u32, String)>) -> impl Responder {
    let text = format!("Hello {}! id:{}", info.1, info.0);
    //HttpResponse::with_body(StatusCode::from_u16(200).unwrap(), text)
    text
}

#[get("/test2")]
async fn hello2(info: web::Path<(u32, String)>) -> impl Responder {
    let text = format!("Hello {}! id:{}", info.1, info.0);
    //HttpResponse::with_body(StatusCode::from_u16(200).unwrap(), text)
    text
}

// http://127.0.0.1:8080/qwe
async fn greet(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    // Все работает на пуле потоков, поэтому остальные запросы смогут отрабатывать нормально
    //std::thread::sleep(std::time::Duration::from_secs(20));

    // Неблокирующий вариант ожидания, который не блокирует поток, а блокирует только данный запрос
    tokio::time::delay_for(std::time::Duration::from_secs(20)).await;

    // Информация о соединении
    let info = req.connection_info();
    println!("{:?}", info);

    // Конфиг приложения
    let config = req.app_config();
    println!("{:?}", config.host());
    println!("{:?}", config.local_addr());
    println!("{:?}", config.secure());

    if let Some(data) = req.app_data::<AppState>() {
        println!("{:?}", data);
    }

    println!("{:?}", data);

    // Мы можем менять общее состояние, но из-за многопоточности - нужно делать синхронизацию
    if let Ok(mut counter) = data.counter.lock(){
        *counter += 1;
    }

    let name = req
        .match_info()
        .get("name")
        .unwrap_or("World");

    format!("Hello {}!", &name)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // Создаем слушателя, чтобы просто переподключаться к открытому сокету при быстром рестарте
    let mut listenfd = ListenFd::from_env();

    let app_lambda = move || {
        //App::new().service(index)
        //web::resource("/{name}/{id}/index.html").to(index)

        let app_data = AppState{
            app_name: "AppSharedData".to_owned(),
            counter: std::sync::Mutex::new(0)
        };

        App::new()
            .data(app_data)
            .route("/", web::get().to(greet))
            .service(
                web::scope("/test/")
                    .service(web::resource("/path1").to(|| async { HttpResponse::Ok() }))
                    .service(web::resource("/path2").route(web::get().to(|| HttpResponse::Ok())))
                    .service(web::resource("/path3").route(web::head().to(|| HttpResponse::MethodNotAllowed()))))
            .route("/{name}", web::get().to(greet))
            .route("/{id}/{name}", web::get().to(greet))
            .service(index) // При определении с помощью макроса - подключать надо таким образом
    };

    // Вариант создания самоподписанного ключа:
    // openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'
    // Вариант с ручной подписью:
    // openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -sha256 -subj "/C=CN/ST=Fujian/L=Xiamen/O=TVlinux/OU=Org/CN=muro.lxd"
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())
        .unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem")
        .unwrap();

    let server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        HttpServer::new(app_lambda).listen(l)?
    } else {
        HttpServer::new(app_lambda)
            .bind("127.0.0.1:8080")?
            .bind_openssl("127.0.0.1:8088", builder)?
    };
    server
        .keep_alive(75) // 75 секунд
        .workers(4) // Можно задать конкретное количество потоков
        .run()
        .await
}