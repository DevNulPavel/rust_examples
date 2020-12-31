use qrcode::{
    QrCode
};
use image::{
    Luma
};
use super::{
    error::{
        QrCodeError
    }
};


pub fn create_qr_data(qr_text: &str) -> Result<Vec<u8>, QrCodeError> {
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
    image::png::PngEncoder::new(png_image_buffer)
        .encode(
            &image_obj, 
            width, 
            height, 
            image::ColorType::L8
        )?;

    Ok(png_image_data)
}
