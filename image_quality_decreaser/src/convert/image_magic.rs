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


pub fn convert_with_image_magic(input_file: &Path, output_file: &Path, quality: u8) -> Result<(), ImageMagicError> {
    // Перегоняем с нужным качеством
    let input_path = input_file.to_str().unwrap();
    let quality_string = format!("{}", quality);
    let out_path = output_file.to_str().unwrap();

    let result = Command::new("convert")
        .args(&[
            input_path,
            "-quality", quality_string.as_str(),
            out_path,
        ])
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