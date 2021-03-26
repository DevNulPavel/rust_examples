use std::{
    path::{
        Path
    },
    process::{
        Command,
        Stdio
    },
    fs::{
        remove_file
    },
    env::{
        temp_dir
    }
};
use lazy_static::{
    lazy_static
};
use regex::{
    Regex
};
use quick_error::{
    ResultExt, 
    quick_error
};


quick_error!{
    #[derive(Debug)]
    pub enum WebPerror{
        /// Ошибка в IO
        IO(context: &'static str, err: std::io::Error){
            context(context: &'static str, err: std::io::Error)-> (context, err)
        }

        /// Утилита завершилась с ошибкой
        ExecutionFailed(err: String){
        }
    }
}

fn get_current_image_size(input_path: &str) -> Result<(u16, u16), WebPerror> {
    // Узнаем исходный размер
    let result = Command::new("webpinfo")
        .args(&[
            input_path
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("webpinfo spawn execution")?
        .wait_with_output()
        .context("webpinfo wait execution")?;

    if result.status.success() {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"Width:\s*(?P<w>[\d]+)[\s\S]*Height:\s*(?P<h>[\d]+)").unwrap();
        }
        
        let out_text = String::from_utf8(result.stdout)
            .expect("webpinfo UTF-8 out parsing failed");

        let captures = RE
            .captures(&out_text)
            .expect("webpinfo output capture failed");
        let cur_width = captures
            .name("w")
            .expect("webpinfo width capture failed")
            .as_str()
            .parse::<u16>()
            .expect("webpinfo width parse failed");
        let cur_height = captures
            .name("h")
            .expect("webpinfo width capture failed")
            .as_str()
            .parse::<u16>()
            .expect("webpinfo height parse failed");

        Ok((cur_width, cur_height))
    }else{
        let err_text = String::from_utf8(result.stderr)
            .expect("webpinfo UTF-8 error parsing failed");
        return Err(WebPerror::ExecutionFailed(format!("webpinfo execution failed: \n{}", err_text)));
    }
}

pub fn convert_webp(input_file: &Path, output_file: &Path, quality: Option<u8>, resize_percent: Option<u16>) -> Result<(), WebPerror> {
    // Сразу же создаем код, который автоматически удаляет временный файлик
    let temporary_file_path = {
        let temporary_file = temp_dir()
            .join(format!("{}.png", uuid::Uuid::new_v4()));

        // Владеющий объект, который на выходе выполняет код ниже
        scopeguard::guard(temporary_file, |temporary_file|{
            if temporary_file.exists() {
                remove_file(temporary_file).ok();
            }
        })
    };
    let temp_file_path_str = temporary_file_path.to_str().unwrap();

    let input_path = input_file.to_str().unwrap();

    // Перегоняем в .png
    {
        let result = Command::new("dwebp")
            .args(&[
                input_path,
                "-o", temp_file_path_str,
                // "-quiet"
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .context("dwebp spawn execution")?
            .wait_with_output()
            .context("dwebp wait execution")?;

        if !result.status.success() {
            let err_text = String::from_utf8(result.stderr)
                .expect("dwebp UTF-8 error parsing failed");
            return Err(WebPerror::ExecutionFailed(format!("dwebp execution failed: \n{}", err_text)));
        }
    }

    // Параметры конвертации файлика
    let out_path = output_file.to_str().unwrap();
    let mut convert_params = vec![
        temp_file_path_str,
        "-o", out_path,
        // "-quiet"
    ];

    // Нужно ли менять качество
    let quality_string;  // Переменная может жить выше, если инициализация и использование ссылки будет ниже
    if let Some(quality) = quality {
        quality_string = format!("{}", quality);
        convert_params.push("-q");
        convert_params.push(quality_string.as_str());
    }

    // Нужно ли делать ресайз?
    let size_w_string;  // Переменная может жить выше, если инициализация и использование ссылки будет ниже
    let size_h_string;  // Переменная может жить выше, если инициализация и использование ссылки будет ниже
    match resize_percent{
        Some(resize_percent) if resize_percent != 100 => {
            // Получаем размер исходной картинки
            let (cur_width, cur_height) = get_current_image_size(input_path)?;

            // Получаем новый размер
            let new_width = ((cur_width as f32) * ((resize_percent as f32) / 100.0)) as u16;
            let new_height = ((cur_height as f32) * ((resize_percent as f32) / 100.0)) as u16;

            // Пишем в параметры
            size_w_string = format!("{w}", w = new_width);
            size_h_string = format!("{h}", h = new_height);
            convert_params.push("-resize");
            convert_params.push(size_w_string.as_str());
            convert_params.push(size_h_string.as_str());
        }
        _ => {
        }
    }

    // Перегоняем в .webp с нужным качеством
    {
        let result = Command::new("cwebp")
            .args(&convert_params)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .context("cwebp spawn execution")?
            .wait_with_output()
            .context("cwebp wait execution")?;

        if !result.status.success() {
            let err_text = String::from_utf8(result.stderr)
                .expect("cwebp UTF-8 error parsing failed");
            return Err(WebPerror::ExecutionFailed(format!("cwebp execution failed: \n{}", err_text)));
        }
    }


    Ok(())
} 