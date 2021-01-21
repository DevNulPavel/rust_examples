use std::{
    io::{
        self
    },
    path::{
        PathBuf,
        Path
    }
};
use serde::{
    Deserialize
};
use serde_json::{
    json
};
use uuid::{
    Uuid
};
use tokio::{
    fs::{
        File,
        remove_file
    },
    io::{
        AsyncRead,
        AsyncReadExt,
        AsyncWrite,
        AsyncWriteExt
    }
};
use futures::{
    Stream,
    StreamExt,
    TryStream,
    TryStreamExt
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
    body::{
        Body
    },
    HttpServer,
    App,
    Responder,
    HttpResponse,
};
use actix_multipart::{
    Multipart
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

async fn upload_image_multipart(mut payload: Multipart) -> Result<HttpResponse, actix_web::Error> {
    // Обходим входные части мультапарта
    // TODO: Может быть просто прерывать работу, а не возвращать ошибку
    while let Some(mut field) = payload.try_next().await? {
        // field.content_type()
        // Получаем тип контента
        /*let content_type = match field.content_disposition(){
            Some(content_type) => content_type,
            None => return Ok(actix_web::HttpResponse::BadRequest().into())
        };
        let filename = match content_type.get_filename() {
            Some(filename) => filename,
            None => return Ok(actix_web::HttpResponse::BadRequest().into())
        };*/

        // TODO: Может ли повторяться
        let filename = Uuid::new_v4().to_string();
    
        // Путь
        let filepath = PathBuf::new()
            .join("/tmp")
            .join(filename);

        // Создаем файлик
        let mut file = File::create(&filepath).await?;

        // TODO: Как-то лучше?
        // Итерируемся по полю и получаем контент файлика
        let mut found_err: Option<actix_web::Error> = None;
        while let Some(chunk) = field.next().await {
            // Данные
            let data = match chunk {
                Ok(data) => data,
                Err(err) => {
                    found_err = Some(err.into());
                    break;
                }
            };

            // Пишем в файлик
            if let Err(err) = file.write_all(&data).await{
                found_err = Some(err.into());
                break;
            }
        }
        // Удаление файлика при ошибке
        if let Some(err) = found_err {
            remove_file(filepath).await?;
            return Err(err);
        }
    }
    Ok(HttpResponse::Ok().into())
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
            .wrap(middleware::Logger::default())
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
                .wrap(middleware::Logger::default())
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