use actix_web::{
    web::{
        self,
        ServiceConfig
    },
    Responder,
    HttpRequest,
    get,
    post
};
use crate::{
    app_state::{
        AppState
    }
};

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

pub fn configure_http_service(cfg: &mut ServiceConfig){
    // Создаем сервис с префиксом /test
    // .guard(guard::Get()) // Обрабатываем только get запросы
    cfg
        .service(index) // Если обработчик сделан на макросе - тогда можно его использовать как сервис
        .service(hello1)
        .service(hello2)
        .route("/", web::route().to(greet));
}