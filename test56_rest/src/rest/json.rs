use serde::{
    Deserialize
};
use log::{
    // info,
    debug,
    // error
};
use actix_web::{
    web::{
        self
    },
    HttpResponse,
};
use crate::{
    ffi::{
        imagemagic_fit_image,
        ImageMagicError
    }
};
use super::{
    response::{
        UploadImageError,
        UploadImageResponse, 
        UploadImageResponseData
    }
};

#[derive(Deserialize)]
pub struct UploadImageJsonData{
    base64_images: Vec<String> // TODO: Читать итерационно для экономии оперативки
}

#[derive(Deserialize)]
pub struct UploadImageJsonUrl{
    urls: Vec<String>
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum UploadImageJson{
    Data(UploadImageJsonData),
    Url(UploadImageJsonUrl)
}

// Кодируем в потоке
#[derive(Debug)]
enum ConverErrLocal {
    Base64(base64::DecodeError),
    Convert(ImageMagicError),
}

async fn convert_base64_data(base64_image: String) -> Result<String, actix_web::Error> {
    // Кодируем в потоке
    let join = actix_web::web::block(move || -> Result<String, ConverErrLocal> {
        // TODO: Не подходит вопросик в конце, поэтому приходится разворачивать в конкретную ошибку
        let data = base64::decode(base64_image)
            .map_err(|err|{
                ConverErrLocal::Base64(err)
            })?;
        let result = imagemagic_fit_image(data, 100, 100)
            .map_err(|err|{
                ConverErrLocal::Convert(err)
            })?;
        Ok(base64::encode(result))
    });
    
    // Получаем ошибку если есть
    let res = join.await?;
    Ok(res)
}

pub async fn upload_image_json(body: web::Json<UploadImageJson>, web_client: web::Data<reqwest::Client>) -> Result<HttpResponse, actix_web::Error> {
    let mut results = Vec::new();
    match body.into_inner() {
        UploadImageJson::Data(base64_info) => {
            for base64_image in base64_info.base64_images.into_iter() {
                // Конвертим
                let result_data = convert_base64_data(base64_image).await?;

                // Вбиваем в результаты
                results.push(UploadImageResponseData{
                    base_64_preview: result_data
                })
            }
        },
        UploadImageJson::Url(url_info) => {
            // TODO: Проверить, что у нас не слишком большой файлик
            for url_str in url_info.urls.into_iter(){
                debug!("Request url: {}", url_str);

                // Запрашиваем
                let response: Result<reqwest::Response, reqwest::Error> = web_client
                    .get(&url_str)
                    .send()
                    .await;  
                    
                let response = match response{
                    Ok(res) => res,
                    Err(err) => {
                        let msg = UploadImageError{
                            message: format!("Invalid response for {}: {}", url_str, err)
                        };
                        return Err(HttpResponse::BadRequest().json(&msg).into());
                    }
                };

                debug!("Request response: {:#?}", response);

                // Какой статус
                if response.status() != http::StatusCode::OK {
                    let msg = UploadImageError{
                        message: format!("Invalid response status ({}) for {}", response.status(), url_str)
                    };
                    return Err(HttpResponse::BadRequest().json(&msg).into());
                }

                // Сначала получаем размер
                let content_length = {
                    let content_length_opt = response
                        .headers()
                        .get("Content-Length")
                        .and_then(|val|{
                            val.to_str().ok()
                        })
                        .and_then(|val|{
                            val.parse::<u64>().ok()
                        });
                    match content_length_opt {
                        Some(length) => length,
                        None => {
                            let msg = UploadImageError{
                                message: format!("Empty content size for {}", url_str)
                            };
                            return Err(HttpResponse::NoContent().json(&msg).into());
                        }
                    }
                };

                debug!("Conent length: {}", content_length);

                // Отбрасываем большие картинки ибо это может быть вообще не картинка
                // Иначе смогут сломать сервер
                if content_length > 1024*1024*16{
                    let msg = UploadImageError{
                        message: format!("Too large content size {} bytes for {}, 16Mb max", content_length, url_str)
                    };
                    return Err(HttpResponse::RangeNotSatisfiable().json(&msg).into());
                }

                // Данные
                let data = match response.bytes().await{
                    Ok(data) => data.to_vec(),
                    Err(err) => {
                        let msg = UploadImageError{
                            message: format!("Body receive failed for {}: {}", url_str, err)
                        };
                        return Err(HttpResponse::RangeNotSatisfiable().json(&msg).into());
                    }
                };

                // Конвертация
                let join = actix_web::web::block(move || -> Result<String, ImageMagicError> {
                    let res = imagemagic_fit_image(data, 100, 100)?;
                    Ok(base64::encode(res))
                });

                let res = join.await?;

                // Вбиваем в результаты
                results.push(UploadImageResponseData{
                    base_64_preview: res
                })
            }
        }
    }
    let data = UploadImageResponse{
        images: results
    };
    Ok(HttpResponse::Ok()
        .json(data))
}