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
use url::{
    Url
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

async fn test_url(base_url: &Url,test_client: &reqwest::Client){
    let body = json!({
        "urls": [
            "https://picsum.photos/200/300"
        ]
    });
    let response = test_client
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
    assert!(data.images.len() > 0);
    assert!(data.images[0].base_64_preview.len() > 0);
}

async fn test_base64(base_url: &Url,test_client: &reqwest::Client){
    let data = tokio::fs::read("test_images/logo.jpg")
        .await
        .expect("File read failed");
    
    let base64_data = base64::encode(data);

    let body = json!({
        "base64_images": [
            base64_data
        ]
    });
    let response = test_client
        .post(base_url.join("upload_image").unwrap())
        .json(&body)
        .send()
        .await
        .expect("Request failed");
    assert_eq!(response.status(), StatusCode::OK);

    // Получаем результат
    let data = response
        .json::<UploadImageResponse>()
        .await
        .expect("Json parse failed");
    assert_eq!(data.images.len(), 1);
    assert!(data.images[0].base_64_preview.len() > 0);

    // Сравнить результат
    let reference_data = tokio::fs::read("test_results/small_logo.jpg")
        .await
        .expect("File read failed");
    assert!(data.images[0].base_64_preview == base64::encode(reference_data));
}

async fn test_multipart(base_url: &Url,test_client: &reqwest::Client){
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

    let response = test_client
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

#[actix_rt::test]
async fn test_server() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init()
        .expect("Logger init failed");

    // Сервер
    let test_server = start_server("127.0.0.1:8888")
        .await
        .expect("Server start error");
    
    // Клиент
    let test_client = reqwest::Client::new();

    let base_url = url::Url::parse("http://localhost:8888")
        .expect("Base url parse failed");

    // Not found
    {
        let response = test_client
            .post(base_url.join("asdad/").unwrap())
            .send()
            .await
            .expect("Request failed");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // Not allowed
    {
        let response = test_client
            .post(base_url.join("upload_image").unwrap())
            .send()
            .await
            .expect("Request failed");
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
    
    // URL
    test_url(&base_url, &test_client).await;

    // Base64
    test_base64(&base_url, &test_client).await;

    // Multipart
    test_multipart(&base_url, &test_client).await;

    info!("All tests finished");

    // Обрываем соединение для успешного завершения
    drop(test_client);
    info!("Client destroyed");

    test_server.stop(true).await;
    info!("Gacefull stop finished");
}