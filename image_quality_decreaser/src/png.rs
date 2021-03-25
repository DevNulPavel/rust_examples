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
use quick_error::{ResultExt, quick_error};


quick_error!{
    #[derive(Debug)]
    pub enum WebPError{
        /// Ошибка в IO
        IO(context: &'static str, err: std::io::Error){
            context(context: &'static str, err: std::io::Error)-> (context, err)
        }

        /// Утилита завершилась с ошибкой
        ExecutionFailed(err: String){
        }
    }
}


pub fn convert_webp(input_file: &Path, output_file: &Path, quality: u8) -> Result<(), WebPError> {
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

    // Перегоняем в .png
    {
        let input_path = input_file.to_str().unwrap();
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
            return Err(WebPError::ExecutionFailed(format!("dwebp execution failed: \n{}", err_text)));
        }
    }

    // Перегоняем в .webp с нужным качеством
    {
        let quality_string = format!("{}", quality);
        let out_path = output_file.to_str().unwrap();
        let result = Command::new("cwebp")
            .args(&[
                "-q", quality_string.as_str(),
                temp_file_path_str,
                "-o", out_path,
                // "-quiet"
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .context("cwebp spawn execution")?
            .wait_with_output()
            .context("cwebp wait execution")?;

        if !result.status.success() {
            let err_text = String::from_utf8(result.stderr)
                .expect("cwebp UTF-8 error parsing failed");
            return Err(WebPError::ExecutionFailed(format!("cwebp execution failed: \n{}", err_text)));
        }
    }


    Ok(())
} 