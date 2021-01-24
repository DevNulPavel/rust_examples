// TODO: Документация


mod ffi;
mod rest;

use std::{
    io::{
        self
    }
};
use log::{
    info,
    debug,
    error
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
    HttpServer,
    App
};
use self::{
    rest::{
        upload_image_multipart,
        upload_image_json
    }
};

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
    use tokio_util::{
        codec::{
            BytesCodec,
            FramedRead
        }
    };
    use reqwest::{
        multipart::{
            Form,
            Part
        },
        Body
    };
    use http::{
        StatusCode
    };
    use serde_json::{
        json
    };
    use crate::{
        rest::{
            UploadImageResponse
        }
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
                .json::<UploadImageResponse>()
                .await
                .expect("Json parse failed");
        }

        // Base64
        {
            let data = tokio::fs::read("test_images/small_building.jpg")
                .await
                .expect("File read failed");
            
            let base64_data = base64::encode(data);

            let body = json!({
                "base64_images": [
                    base64_data
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
                .json::<UploadImageResponse>()
                .await
                .expect("Json parse failed");
            assert_eq!(data.images.len(), 1);
            assert!(data.images[0].base_64_preview.len() > 0);
            // TODO: Сравнить результат
        }

        // Multipart
        {
            let body1 = {
                let file = tokio::fs::File::open("test_images/airplane.png")
                    .await
                    .expect("Test image is not found");
                let reader = FramedRead::new(file, BytesCodec::default());
                Body::wrap_stream(reader)
            };
            let body2 = {
                let file = tokio::fs::File::open("test_images/airplane.png")
                    .await
                    .expect("Test image is not found");
                let reader = FramedRead::new(file, BytesCodec::default());
                Body::wrap_stream(reader)
            };

            let form = Form::default()
                .part("first", Part::stream(body1))
                .part("second", Part::stream(body2));

            let response = client
                .post(base_url.join("upload_image").unwrap())
                .multipart(form)
                .send()
                .await
                .expect("Request failed");
            assert_eq!(response.status(), StatusCode::OK);

            let data = response
                .json::<UploadImageResponse>()
                .await
                .expect("Json parse failed");
            assert_eq!(data.images.len(), 2);
            assert!(data.images[0].base_64_preview.len() > 0);
            assert!(data.images[1].base_64_preview.len() > 0);

            // TODO: сравнение данных у картинки
        }

        info!("All tests finished");

        // Обрываем соединение для успешного завершения
        drop(client);
        info!("Client destroyed");

        test_server.stop(true).await;
        info!("Gacefull stop finished");
    }
}