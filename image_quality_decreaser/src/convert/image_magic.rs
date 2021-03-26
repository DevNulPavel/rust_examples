use std::{
    path::{
        Path
    },
    process::{
        Command,
        Stdio
    }
};
use quick_error::{
    ResultExt, 
    quick_error
};


quick_error!{
    #[derive(Debug)]
    pub enum ImageMagicError{
        /// Ошибка в IO
        IO(context: &'static str, err: std::io::Error){
            context(context: &'static str, err: std::io::Error)-> (context, err)
        }

        /// Утилита завершилась с ошибкой
        ExecutionFailed(err: String){
        }
    }
}


pub fn convert_with_image_magic(input_file: &Path, output_file: &Path, quality: Option<u8>, resize_percent: Option<u16>) -> Result<(), ImageMagicError> {
    // Перегоняем с нужным качеством
    let mut convert_params = Vec::new();
    convert_params.reserve(8);

    // Входное значение
    let input_path = input_file.to_str().unwrap();
    convert_params.push(input_path);

    // Нужно ли менять качество
    let quality_string;  // Переменная может жить выше, если инициализация и использование ссылки будет ниже
    if let Some(quality) = quality {
        quality_string = format!("{}", quality);
        convert_params.push("-quality");
        convert_params.push(quality_string.as_str());
    }

    // Нужно ли делать ресайз?
    let percent_string;
    match resize_percent{
        Some(resize_percent) if resize_percent != 100 => {
            percent_string = format!("{}%", resize_percent);
            convert_params.push("-resize");
            convert_params.push(percent_string.as_str());
        }
        _ => {
        }
    }

    // Результат
    let out_path = output_file.to_str().unwrap();
    convert_params.push(out_path);

    let result = Command::new("convert")
        .args(&convert_params)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("ImageMagic's 'convert' spawn execution")?
        .wait_with_output()
        .context("ImageMagic's 'convert' wait execution")?;

    if !result.status.success() {
        let err_text = String::from_utf8(result.stderr)
            .expect("ImageMagic's UTF-8 error parsing failed");
        return Err(ImageMagicError::ExecutionFailed(format!("ImageMagic's execution failed: \n{}", err_text)));
    }

    Ok(())
} 