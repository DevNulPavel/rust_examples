use std::{
    io::{
        self
    },
    sync::{
        Arc
    },
    path::{
        Path
    }
};
use log::{
    info,
    debug,
    error
};
use serde::{
    Deserialize
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
    guard::{
        self
    },
    client::{
        Client
    },
    HttpServer,
    HttpResponse,
    App
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
struct AuthRequest{
    userid: String,
    sessionkey: String,
    sessionid: String,
    lang: String,
    platform: String,

    #[serde(rename = "platformSPIL")]
    platform_spil: String
}

async fn auth(params: web::Query<AuthRequest>, shared_data: web::Data<SharedAppData>) -> Result<HttpResponse, actix_web::Error> {
    debug!("Auth request with params: {:#?}", params.0);
    let buffer = {
        let mut buffer = String::new();
        buffer.push_str(&params.userid);
        buffer.push_str(&params.sessionkey);
        buffer.push_str(&shared_data.secret_key);
        buffer    
    };
    let result = format!("{:x}", md5::compute(buffer));
    if params.sessionid.eq(&result) {
        Ok(HttpResponse::Ok().finish())
    }else{
        Ok(HttpResponse::Forbidden().reason("Invalid hash").finish())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize)]
struct PaymentInfoRequest{
    userid: String,
    transactionId: String,
    signature: String
}

async fn get_payment_info(params: web::Form<PaymentInfoRequest>) -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().finish())
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize)]
struct ProcessPayRequest{
    userid: String,
    game: String,
    ts: String,
    coins: String,
    price: String,
    currency: String,
    tid: String,
    oid: String,
    signature: String
}

async fn process_pay(body: web::Form<PaymentInfoRequest>) -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().finish())
}

////////////////////////////////////////////////////////////////////////////////

async fn static_forward(req: web::HttpRequest, 
                        body: web::Bytes, 
                        target_url: web::Data<Url>, 
                        client: web::Data<Client>) -> Result<HttpResponse, actix_web::Error> {

    let mut new_url = target_url.get_ref().clone();
    new_url.set_path(req.uri().path());
    new_url.set_query(req.uri().query());

    // TODO: This forwarded implementation is incomplete as it only handles the inofficial
    // X-Forwarded-For header but not the official Forwarded one.
    let forwarded_req = client
        .request_from(new_url.as_str(), req.head())
        .no_decompress();
    let forwarded_req = if let Some(addr) = req.head().peer_addr {
        forwarded_req.header("x-forwarded-for", format!("{}", addr.ip()))
    } else {
        forwarded_req
    };

    let mut res = forwarded_req.send_body(body).await.map_err(actix_web::Error::from)?;

    let mut client_resp = HttpResponse::build(res.status());
    // Remove `Connection` as per
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    for (header_name, header_value) in
        res.headers().iter().filter(|(h, _)| *h != "connection")
    {
        client_resp.header(header_name.clone(), header_value.clone());
    }

    Ok(client_resp.body(res.body().await?))
}

////////////////////////////////////////////////////////////////////////////////

struct SharedAppData{
    secret_key: String
}

////////////////////////////////////////////////////////////////////////////////

async fn start_server(addr: &str) -> io::Result<dev::Server>{
    let static_redirect_url = Url::parse("http://127.0.0.0:8080").unwrap();

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
            .route(web::route().to(static_forward));
        
        App::new()
            .wrap(middleware::Logger::default())
            .default_service(web::route().to(|| { web::HttpResponse::NotFound() }))
            .service(auth_service)
            .service(get_payment_info_service)
            .service(process_pay_service)
            .service(static_files_service)
            .app_data(shared_data.clone()) // Можно получить у запроса
            .app_data(Client::new()) // Можно получить у запроса
            .app_data(static_redirect_url.clone()) // Можно получить у запроса
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

    let server = start_server("0.0.0.0:8888").await?;
    info!("Server started");
    
    tokio::signal::ctrl_c().await.expect("Signal wait failed");
    info!("Stop signal received");

    server.stop(true).await;
    info!("Gracefull stop finished");

    Ok(())
}