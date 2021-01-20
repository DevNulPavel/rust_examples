use std::{
    io::{
        self
    }
};
use serde::{
    Deserialize
};
use serde_json::{
    json
};
use actix_web::{
    dev::{
        self
    },
    web::{
        self
    },
    guard::{
        self
    },
    body::{
        Body
    },
    HttpServer,
    App,
    Responder,
    HttpResponse,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize)]
struct UploadImageQuery{
}
#[derive(Deserialize)]
struct UploadImageJsonData{
    base64: String
}
#[derive(Deserialize)]
struct UploadImageJsonUrl{
    url: String
}
#[derive(Deserialize)]
#[serde(untagged)]
enum UploadImageJson{
    Data(UploadImageJsonData),
    Url(UploadImageJsonUrl)
}
async fn upload_image_json(query: web::Query<UploadImageQuery>, 
                           body: web::Json<UploadImageJson>) -> impl Responder {
    HttpResponse::Ok()
        .finish()
}

////////////////////////////////////////////////////////////////////////////////

fn build_image_service() -> impl dev::HttpServiceFactory {
    let upload_json_route = web::route()
        .guard(guard::Post())
        .guard(guard::Header("Content-Type", "application/json"))
        .to(upload_image_json);

    let image_service = web::resource("/upload_image")
        .route(upload_json_route);
    
    image_service
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    // Важно! На каждый поток у нас создается свое приложение со своими данными
    let app_builder = ||{
        let image_service = build_image_service();
        App::new()
            .default_service(web::route().to(|| { web::HttpResponse::NotFound() }))
            .service(image_service)
            // .app_data() // Можно получить у запроса
            // .data(data) // Можно прокидывать как параметр у обработчика
    };

    // Запускаем сервер
    HttpServer::new(app_builder)
        .bind("0.0.0.0:8080")?
        .keep_alive(75_usize) // 75 секунд
        .run()
        .await
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{
        *
    };
    use actix_web::{
        test::{
            self
        }
    };
    use http::StatusCode;

    #[actix_rt::test]
    async fn test_server() {
        // TODO: Приходится дублировать код, так как не вынести в функцию из-за закрытых структур
        let app_builder = ||{
            let image_service = build_image_service();
            App::new()
                .default_service(web::route().to(|| { web::HttpResponse::NotFound() }))
                .service(image_service)
                // .app_data() // Можно получить у запроса
                // .data(data) // Можно прокидывать как параметр у обработчика
        };
        let server = test::start(app_builder);

        // Not found
        {
            let response = server
                .post("/adads")
                .send()
                .await
                .expect("Request failed");
            assert_eq!(response.status(), StatusCode::NOT_FOUND);
        }

        // Not allowed
        {
            let response = server
                .post("/upload_image")
                .send()
                .await
                .expect("Request failed");
            assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        }
        
        // URL
        {
            let body = json!({
                "url": "https://picsum.photos/200/300"
            });
            let response = server
                .post("/upload_image")
                .send_json(&body)
                .await
                .expect("Request failed");
            assert_eq!(response.status(), StatusCode::OK);
        }

        // Base64
        {
            let body = json!({
                "base64": "asddsadsa"
            });
            let response = server
                .post("/upload_image")
                .send_json(&body)
                .await
                .expect("Request failed");
            assert_eq!(response.status(), StatusCode::OK);
        }
    }
}