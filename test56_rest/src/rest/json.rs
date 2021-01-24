use serde::{
    Deserialize
};
use log::{
    info,
    debug,
    error
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

pub async fn upload_image_json(body: web::Json<UploadImageJson>) -> Result<HttpResponse, actix_web::Error> {
    let mut results = Vec::new();
    match body.into_inner() {
        UploadImageJson::Data(base64_info) => {
            for base64_image in base64_info.base64_images {
                // Кодируем в потоке
                #[derive(Debug)]
                enum ErrLocal{
                    Base64(base64::DecodeError),
                    Convert(ImageMagicError)
                }
                let join = actix_web::web::block(move || -> Result<Vec<u8>, ErrLocal> {
                    // TODO: Не подходит вопросик в конце, поэтому приходится разворачивать в конкретную ошибку
                    let data = base64::decode(base64_image)
                        .map_err(|err|{
                            ErrLocal::Base64(err)
                        })?;
                    let result = imagemagic_fit_image(data, 100, 100)
                        .map_err(|err|{
                            ErrLocal::Convert(err)
                        })?;
                    Ok(result)
                });
                // Получаем ошибку если есть
                let result_data = join.await?;
                // Вбиваем в результаты
                results.push(UploadImageResponseData{
                    base_64_preview: base64::encode(result_data)
                })
            }
        },
        UploadImageJson::Url(url_info) => {

        }
    }
    let data = UploadImageResponse{
        images: results
    };
    Ok(HttpResponse::Ok()
        .json(data))
}