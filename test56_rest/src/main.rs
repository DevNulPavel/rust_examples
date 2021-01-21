use std::{
    io::{
        self
    },
    path::{
        PathBuf,
        Path
    }
};
use serde::{Deserialize, Serialize, __private::ser};
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
use log::{
    info,
    debug,
    error
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
        Body,
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

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize))]
struct ImageResponse{
    id: String
}
#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize))]
struct UploadResponse{
    images: Vec<ImageResponse>
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize)]
struct UploadImageQuery{
}
#[derive(Deserialize)]
struct UploadImageJsonData{
    base64_images: Vec<String> // TODO: Читать итерационно для экономии оперативки
}
#[derive(Deserialize)]
struct UploadImageJsonUrl{
    urls: Vec<String>
}
#[derive(Deserialize)]
#[serde(untagged)]
enum UploadImageJson{
    Data(UploadImageJsonData),
    Url(UploadImageJsonUrl)
}
async fn upload_image_json(query: web::Query<UploadImageQuery>, 
                           body: web::Json<UploadImageJson>) -> impl Responder {

    let data = UploadResponse{
        images: vec![]
    };
    HttpResponse::Ok()
        .json(data)
}


////////////////////////////////////////////////////////////////////////////////

async fn upload_image_multipart(mut payload: Multipart) -> Result<HttpResponse, actix_web::Error> {
    // Обходим входные части мультапарта
    // TODO: Может быть просто прерывать работу, а не возвращать ошибку
    while let Some(mut field) = payload.try_next().await? {
        {
            let disposition = match field.content_disposition(){
                Some(disposition) => disposition,
                None => return Ok(actix_web::HttpResponse::BadRequest().into())
            };
            if !disposition.is_form_data(){
                return Ok(actix_web::HttpResponse::BadRequest().into())
            }
        }

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
    let data = UploadResponse{
        images: vec![]
    };
    Ok(HttpResponse::Ok()
        .json(data))
}

////////////////////////////////////////////////////////////////////////////////

fn build_image_service() -> impl dev::HttpServiceFactory {
    let upload_json_route = web::route()
        .guard(guard::Post())
        .guard(guard::Header("Content-Type", "application/json"))
        .to(upload_image_json);

    let upload_mulipart_route = web::route()
        .guard(guard::Post())
        .guard(guard::fn_guard(|req|{
            if let Some(val) = req.headers.get("Content-Type") {
                if let Ok(val_str) = val.to_str(){
                    return val_str.starts_with("multipart/form-data");
                }
            }
            false
        }))
        .to(upload_image_multipart);

    let image_service = web::resource("/upload_image")
        .route(upload_json_route)
        .route(upload_mulipart_route);
    
    image_service
}

////////////////////////////////////////////////////////////////////////////////

async fn start_server(addr: &str) -> io::Result<dev::Server>{
    // TODO: Логирование

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
    let server = HttpServer::new(app_builder)
        .bind(addr)?
        .keep_alive(75_usize) // 75 секунд
        .run();

    Ok(server)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Debug) // TODO: ???
        .try_init()
        .expect("Logger init failed"); // TODO: ???

    let server = start_server("0.0.0.0:8080").await?;
    info!("Server started");
    
    tokio::signal::ctrl_c().await.expect("Signal wait failed");
    info!("Stop signal received");

    server.stop(true).await;
    info!("Gracefull stop finished");

    Ok(())
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
    use http::{
        StatusCode
    };

    #[actix_rt::test]
    async fn test_server() {
        pretty_env_logger::formatted_builder()
            .filter_level(log::LevelFilter::Debug)
            // .is_test(true)
            .try_init()
            .expect("Logger init failed");
        // TODO: Стандартный тестовый сервер в actix-web не очень удобный

        // TODO: остановка сервера в тестах
        let test_server = start_server("127.0.0.1:8888")
            .await
            .expect("Server start error");

        let client = reqwest::Client::new();

        let base_url = url::Url::parse("http://localhost:8888").expect("Base url parse failed");

        // Not found
        {
            let response = client
                .post(base_url.join("asdad/").unwrap())
                .send()
                .await
                .expect("Request failed");
            assert_eq!(response.status(), StatusCode::NOT_FOUND);
        }

        // Not allowed
        {
            let response = client
                .post(base_url.join("upload_image").unwrap())
                .send()
                .await
                .expect("Request failed");
            assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        }
        
        // URL
        {
            let body = json!({
                "urls": [
                    "https://picsum.photos/200/300"
                ]
            });
            let response = client
                .post(base_url.join("upload_image").unwrap())
                .json(&body)
                .send()
                .await
                .expect("Request failed");
            assert_eq!(response.status(), StatusCode::OK);

            let data = response
                .json::<UploadResponse>()
                .await
                .expect("Json parse failed");
        }

        // Base64
        {
            let body = json!({
                "base64_images": [
                    "asddsadsa"
                ]
            });
            let response = client
                .post(base_url.join("upload_image").unwrap())
                .json(&body)
                .send()
                .await
                .expect("Request failed");
            assert_eq!(response.status(), StatusCode::OK);
        }

        // Multipart
        {
            let form = reqwest::multipart::Form::default()
                .part("first", reqwest::multipart::Part::text("asdsd"))
                .part("second", reqwest::multipart::Part::text("asdsd"));

            let response = client
                .post(base_url.join("upload_image").unwrap())
                .multipart(form)
                .send()
                .await
                .expect("Request failed");
            assert_eq!(response.status(), StatusCode::OK);
        }
        info!("All tests finished");

        // Обрываем соединение для успешного завершения
        drop(client);
        info!("Client destroyed");

        test_server.stop(true).await;
        info!("Gacefull stop finished");
        // test_server.await.expect("Server stop failed");
    }
}