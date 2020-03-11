use qrcode::QrCode;
use image::Luma;
use crate::errors::MessageError;


pub async fn send_qr_to_channel(client: &reqwest::Client, api_token: &str, channel: &str, qr_text: &str, qr_commentary: &str) -> Result<(), MessageError>{   
    let image_data: Vec<u8> = {
        // Encode some data into bits.
        let code = QrCode::new(qr_text.as_bytes())?; // Конвертация ошибки произойдет автоматически

        // Рендерим картинку
        let image_obj = code.render::<Luma<u8>>().build();

        // Ширина и высота
        let width = image_obj.width();
        let height = image_obj.height();

        // Фактический вектор с данными
        let mut png_image_data: Vec<u8> = Vec::new();

        // Создаем курсор на мутабельный вектор, курсор
        let png_image_data_cursor = std::io::Cursor::new(&mut png_image_data);

        // Создаем буффер с мутабельной ссылкой на вектор
        // Можно сразу передавать &mut png_image_data вместо курсора, но с курсором нагляднее
        let png_image_buffer = std::io::BufWriter::with_capacity(2048, png_image_data_cursor);

        // Конвертим
        image::png::PNGEncoder::new(png_image_buffer).encode(
                &image_obj, 
                width, 
                height, 
                image::ColorType::Gray(8))
            .map_err(|err|{
                MessageError::QRFileDidNotConvertToPng(err)
            })?;

        png_image_data
    };

    // File path
    let new_uuid = uuid::Uuid::new_v4();
    let filename = format!("{}.png", new_uuid);

    // Есть или нет комментарий?
    let commentary = match qr_commentary.len() {
        0 => qr_commentary,
        _ => qr_text
    };

    use reqwest::multipart::Part;
    use reqwest::multipart::Form;

    let form = Form::new()
        .part("channels", Part::text(channel.to_owned()))
        .part("initial_comment", Part::text(commentary.to_owned()))
        .part("filename", Part::text(filename.to_owned()))
        //.part("file", Part::bytes(file_data).file_name(filename.to_owned()));
        .part("file", Part::stream(image_data).file_name(filename.to_owned()));

    client.post("https://slack.com/api/files.upload")
        .bearer_auth(api_token)
        .multipart(form)
        .send()
        .await?;

    Ok(())
}
