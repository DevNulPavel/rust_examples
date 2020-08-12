use actix_web::{
    web::{
        self,
        ServiceConfig
    },
    Responder,
    HttpRequest,
    HttpResponse,
    get,
    //post
};
use actix_session::{
    Session
};
use serde::{
    Deserialize
};
use crate::{
    app_state::{
        AppState
    }
};

#[derive(Deserialize)]
struct GetParams{
    id: u32,
    name: String
}

// http://127.0.0.1:8080/
// Можно описывать путь с помощью макроса
#[get("/")]
async fn index(info: web::Path<(u32, String)>) -> impl Responder {
    let text = format!("Hello {}! id:{}", info.1, info.0);
    //HttpResponse::with_body(StatusCode::from_u16(200).unwrap(), text)
    text
}

#[get("/test1/{id}/{name}")]
async fn hello1(req: web::HttpRequest) -> impl Responder {
    // Получаем id как Option
    let id = match req.match_info().get("id"){
        Some(id) => {
            id
        },
        None => {
            return String::from("id parse failed");
        }
    };
    // Получаем name в виде пустой строки если нету
    let name = req.match_info().query("name");

    let text = format!("Hello {}! id:{}", name, id);
    //HttpResponse::with_body(StatusCode::from_u16(200).unwrap(), text)
    
    text
}

// 127.0.0.1:8080/test2/1/qwe
#[get("/test2/{id}/{name}")]
async fn hello2(info: web::Path<GetParams>) -> impl Responder {
    let text = format!("Hello {}! id:{}", info.name, info.id);
    //HttpResponse::with_body(StatusCode::from_u16(200).unwrap(), text)
    text
}

/// extract form data using serde
/// this handler gets called only if the content type is *x-www-form-urlencoded*
/// and the content of the request could be deserialized to a `FormData` struct
#[get("/test3/{id}/{name}")]
async fn hello3(form: web::Form<GetParams>) -> impl Responder {
    format!("Welcome {}!", form.id)
}

// http://127.0.0.1:8080/greet
async fn greet(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    // Все работает на пуле потоков, поэтому остальные запросы смогут отрабатывать нормально
    //std::thread::sleep(std::time::Duration::from_secs(20));

    // Неблокирующий вариант ожидания, который не блокирует поток, а блокирует только данный запрос
    //tokio::time::delay_for(std::time::Duration::from_secs(1)).await;

    // Информация о соединении
    let info = req.connection_info();
    println!("{:?}", info);

    // Конфиг приложения
    let config = req.app_config();
    println!("{:?}", config.host());
    println!("{:?}", config.local_addr());
    println!("{:?}", config.secure());

    // Получаем данные всего приложения из запроса помимо параметра
    if let Some(app_data) = req.app_data::<AppState>() {
        println!("{:?}", app_data);
    }

    // Данные запроса
    println!("{:?}", data);

    // Мы можем менять общее состояние, но из-за многопоточности - нужно делать синхронизацию
    if let Ok(mut counter) = data.counter.lock(){
        *counter += 1;
    }

    // Получаем параметр из get запроса
    let name = req
        .match_info()
        .get("name")
        .unwrap_or("World");

    format!("Hello {}!", &name)
}

async fn authorized_handler(session: Session) -> Result<HttpResponse, actix_web::Error> {
    // access session data
    if let Some(count) = session.get::<i32>("counter")? {
        if count < 5{
            session.set("counter", count + 1)?;
        }else{
            session.remove("counter");
            return Ok(HttpResponse::PermanentRedirect()
                .reason("Counter greater 5")
                .body("Counter greater 5, redirect!"));
        }
    } else {
        session.set("counter", 1)?;
    }


    Ok(HttpResponse::Ok().body(format!(
        "Count is {:?}!",
        session.get::<i32>("counter")?.unwrap()
    )))
}

async fn login() -> impl Responder {
    "Login page"
}

pub fn configure_http_service(cfg: &mut ServiceConfig){
    // Создаем сервис с префиксом /test
    // .guard(guard::Get()) // Обрабатываем только get запросы

    cfg
        .service(index) // Если обработчик сделан на макросе - тогда можно его использовать как сервис
        .service(hello1)
        .service(hello2)
        .route("/authorized", web::get().to(authorized_handler)) // Внутри создается ресурс и сервис
        .route("/greet", web::route().to(greet))
        .route("/login", web::route().to(login)); // Внутри создается ресурс и сервис
}