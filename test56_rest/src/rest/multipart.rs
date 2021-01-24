use log::{
    info,
    debug,
    error
};
use futures::{
    StreamExt,
    TryStreamExt
};
use actix_web::{
    HttpResponse,
};
use actix_multipart::{
    Multipart,
    Field
};
use crate::{
    ffi::{
        imagemagic_fit_image,
        ImageMagicError
    }
};
use super::{
    response::{
        UploadImageResponseData,
        UploadImageResponse
    }
};

/*async fn save_multipart_to_file(mut field: Field) -> Result<PathBuf, actix_web::Error> {
    // TODO: Может ли повторяться
    let filename = Uuid::new_v4().to_string();
    
    // Путь
    // Если нету директории временных файлов - лучше сразу крашить
    let filepath = PathBuf::new()
        .join(dirs::template_dir().expect("There is no template directory")) // TODO: ошибка?
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

    Ok(filepath)
}*/

async fn read_multipart_to_vec(mut field: Field) -> Result<Vec<u8>, actix_web::Error> {
    // Есть размер?
    let data_size = field
        .headers()
        .get("Content-Length")
        .and_then(|val|{
            val.to_str().ok()
        })
        .and_then(|val|{
            val.parse::<usize>().ok()
        });

    // Конечный буффер с резервированием если есть инфа
    let mut result_buffer = if let Some(size) = data_size {
        Vec::with_capacity(size)
    } else{
        Vec::new()
    };
    
    // TODO: Как-то лучше?
    // Итерируемся по полю и получаем контент файлика
    while let Some(chunk) = field.next().await {
        // Данные
        let data = chunk?;
        // Пишем в буффер
        result_buffer.extend(data.into_iter());
    }

    Ok(result_buffer)
}

// TODO: Расширение картинки
/*let file_extention = {
    let disposition = match field.content_disposition(){
        Some(disposition) => disposition,
        None => return Ok(actix_web::HttpResponse::BadRequest().into())
    };
    if !disposition.is_form_data(){
        return Ok(actix_web::HttpResponse::BadRequest().into())
    }
    let filename = match disposition.get_filename() {
        Some(filename) => filename,
        None => return Ok(actix_web::HttpResponse::BadRequest().into())
    };
    let ext = Path::new(filename)
        .extension()
        .and_then(|ext| {
            ext.to_str()
        })
        .map(|ext|{
            ext.to_owned()
        });
    ext
};*/

pub async fn upload_image_multipart(mut payload: Multipart) -> Result<HttpResponse, actix_web::Error> {
    // Обходим входные части мультапарта
    // TODO: Может быть просто прерывать работу, а не возвращать ошибку
    let mut results = vec![];
    while let Some(field) = payload.try_next().await? {
        // Верные ли данные
        let disposition = match field.content_disposition(){
            Some(disposition) => disposition,
            None => return Ok(actix_web::HttpResponse::BadRequest().into())
        };
        if !disposition.is_form_data(){
            return Ok(actix_web::HttpResponse::BadRequest().into())
        }

        // Читаем
        let buffer = read_multipart_to_vec(field).await?;

        // Конвертируем на пуле потоков, чтобы не блокироваться
        let preview = actix_web::web::block(move || -> Result<String, ImageMagicError> {
            let data = imagemagic_fit_image(buffer, 100, 100)?;
            // Base64
            let base_64_data = base64::encode::<Vec<u8>>(data);
            Ok(base_64_data)
        }).await?;

        // Результат
        results.push(UploadImageResponseData{
            base_64_preview: preview
        })
    }
    let data = UploadImageResponse{
        images: results
    };
    Ok(HttpResponse::Ok()
        .json(data))
}

// TODO: Логирование