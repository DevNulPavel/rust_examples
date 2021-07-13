mod app_arguments;
mod helpers;
mod pvr;
mod types;

use crate::{app_arguments::AppArguments, helpers::create_dir_for_file, types::ImageSize, pvr::pvrgz_image_size};
use eyre::{ContextCompat, WrapErr};
// use fallible_iterator::{FallibleIterator, FromFallibleIterator, IntoFallibleIterator};
// use rayon::prelude::*;
use serde::Deserialize;
use std::{
    convert::TryInto,
    ffi::OsStr,
    fs::File,
    fmt::Write,
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use tracing::{debug, instrument, Level};
use walkdir::WalkDir;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Настойка уровня логирования
#[instrument(level = "error", skip(arguments))]
fn setup_logging(arguments: &AppArguments) -> Result<(), eyre::Error> {
    use tracing_subscriber::prelude::*;

    // Настройка логирования на основании количества флагов verbose
    let level = match arguments.verbose {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        3 => Level::TRACE,
        _ => {
            return Err(eyre::eyre!("Verbose level must be in [0, 3] range"));
        }
    };
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::from_level(level))
        .with(tracing_subscriber::filter::EnvFilter::new(env!("CARGO_PKG_NAME"))) // Логи только от текущего приложения, без библиотек
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_error::ErrorLayer::default()) // Для поддержки захватывания SpanTrace в eyre
        .try_init()
        .wrap_err("Tracing init failed")?;

    Ok(())
}

/// Выполняем валидацию переданных аргументов приложения
#[instrument(level = "error", skip(arguments))]
fn validate_arguments(arguments: &AppArguments) -> Result<(), eyre::Error> {
    eyre::ensure!(
        arguments.source_directory.exists(),
        "Source dragonbones directory does not exist: {:?}",
        arguments.source_directory
    );

    Ok(())
}

// Попытка получения размера файлика
#[instrument(level = "error")]
fn try_get_image_size(path: &Path) -> Result<ImageSize, eyre::Error> {
    // Получаем расширение с нижнем регистре, либо выходим с None
    let file_ext = match path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_lowercase()) {
        Some(ext) => ext,
        None => return Err(eyre::eyre!("File extention is missing, path = {:?}", &path)),
    };

    match file_ext.as_str() {
        "jpg" | "png" | "webp" | "jpeg" => {
            // Получаем размер картинки для файлика
            let size = imagesize::size(path)?;
            Ok(ImageSize {
                width: size.width as u32,
                height: size.height as u32,
            })
        }
        "pvrgz" => {
            // Нашим собстенным способом читаем размер картинки из PVRGZ
            let res = pvrgz_image_size(path).wrap_err("Pvrgz convert")?;
            Ok(res)
        }
        _ => Err(eyre::eyre!("Unsupported file extention, path = {:?}", &path)),
    }
}

enum ValidateResult {
    Valid,
    Invalid { image_name: String },
}

#[instrument(level = "error")]
fn validate_json_file(json_file_path: &Path) -> Result<ValidateResult, eyre::Error> {
    let json_file = File::open(json_file_path)?;

    #[derive(Debug, Deserialize)]
    struct TextureInfo {
        width: Option<i32>,
        height: Option<i32>,
        #[serde(rename = "imagePath")]
        image_path: String,
    }
    let json_data: TextureInfo = serde_json::from_reader(json_file).wrap_err("Deserealize json error")?;

    // Есть все указанные поля?
    let with_size = json_data.height.is_some() && json_data.width.is_some();

    if with_size {
        Ok(ValidateResult::Valid)
    } else {
        Ok(ValidateResult::Invalid {
            image_name: json_data.image_path,
        })
    }
}

#[instrument(level = "error")]
fn execute_app() -> Result<(), eyre::Error> {
    // Аргументы коммандной строки
    let arguments = app_arguments::AppArguments::from_args_safe().wrap_err("Arguments parsing")?;

    // Настройка логирования на основании количества флагов verbose
    setup_logging(&arguments).wrap_err("Logging setup")?;

    // Display arguments
    debug!(?arguments, "App arguments");

    // Валидация параметров приложения
    validate_arguments(&arguments).wrap_err("Validate arguments")?;

    // TODO: Однопоточный вариант вроде бы как работает быстро, то есть можно обойтись и без rayon
    // Вариант с rayon даже медленнее работал

    // Метабельная переменная для строк ошибок
    let mut err_lines = String::new();
    
    // Обходим все вхождения
    for entry in WalkDir::new(&arguments.source_directory).into_iter() {
        let path = entry?.into_path();

        // Перегоняем в utf-8 строку
        let path_str = path
            .to_str()
            .ok_or_else(|| eyre::eyre!("Failed convert path to utf-8 string, path = {:?}", &path))?;

        // Проверяем имя
        if !path_str.ends_with("texture.json") && !path_str.ends_with("_tex.json") {
            continue;
        }

        // Это вообще файлик?
        if !path.is_file() {
            continue;
        }

        // Проверяем валидность
        let status = validate_json_file(&path)?;

        // Если файлик невалидный
        if let ValidateResult::Invalid { image_name } = status {
            let image_path = path.with_file_name(image_name);
            
            // Узнаем необходимый размер текстуры
            let texture_size = try_get_image_size(&image_path).wrap_err("Image size request")?;

            // Пишем сообщение с ошибкой
            writeln!(&mut err_lines, "- {}, valid size = {{width: {}, height: {}}}", path_str, texture_size.width, texture_size.height)?;
        }
    }

    // Если были найдены такие файлики
    if err_lines.len() > 0 {
        use ansi_term::Color::Red;
        eprintln!("{}\n{}", Red.paint("Invalid dragonbones files found:"), err_lines);

        // Просто завершаем наше приложение с кодом ошибки
        std::process::exit(2);
    }

    Ok(())
}

fn main() {
    // Человекочитаемый вывод паники
    color_backtrace::install();

    // Настройка color eyre для ошибок
    color_eyre::install().expect("Error setup failed");

    // Запускаем наш код и обрабатываем ошибку если надо
    if let Err(err) = execute_app() {
        // При ошибке не паникуем, а спокойно выводим сообщение и завершаем приложение с кодом ошибки
        // Это нужно для того, чтобы вывести содержимое ошибки, а не получать новый стектрейс из паники
        eprint!("Error! Failed with: {:?}", err);
        std::process::exit(1);
    }
}
