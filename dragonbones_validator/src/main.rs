mod app_arguments;
mod helpers;
mod pvr;
mod types;

use crate::{app_arguments::AppArguments, pvr::pvrgz_image_size, types::ImageSize};
use eyre::{ContextCompat, WrapErr};
use serde::Deserialize;
use std::{fmt::Write, fs::File, path::Path, io::BufReader};
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
        arguments.json_files_directory.exists(),
        "Source dragonbones directory does not exist: {:?}",
        arguments.json_files_directory
    );
    if let Some(alternative_texture_files_directory) = &arguments.alternative_texture_files_directory {
        eyre::ensure!(
            alternative_texture_files_directory.exists(),
            "Alternative textures directory does not exist: {:?}",
            alternative_texture_files_directory
        );
    }

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
        _ => Err(eyre::eyre!("Unsupported image file extention, path = {:?}", &path)),
    }
}

enum ValidateResult {
    Valid,
    NoImageName,
    InvalidSize { image_name: String },
}

#[instrument(level = "error")]
fn validate_json_file(json_file_path: &Path) -> Result<ValidateResult, eyre::Error> {
    let json_file = File::open(json_file_path)?;

    #[derive(Debug, Deserialize)]
    struct TextureInfo {
        width: Option<i32>,
        height: Option<i32>,
        #[serde(rename = "imagePath")]
        image_path: Option<String>,
    }
    let json_data: TextureInfo = serde_json::from_reader(BufReader::new(json_file)).wrap_err("Deserealize json error")?;

    match json_data.image_path {
        Some(image_path) => {
            // Есть все указанные поля?
            let with_size = json_data.height.is_some() && json_data.width.is_some();

            if with_size {
                Ok(ValidateResult::Valid)
            } else {
                Ok(ValidateResult::InvalidSize { image_name: image_path })
            }
        }
        None => Ok(ValidateResult::NoImageName),
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
    for entry in WalkDir::new(&arguments.json_files_directory).into_iter() {
        // Перегоним в путь
        let full_json_file_path = entry.wrap_err("Directory enter error")?.into_path();

        // Относительный путь к json файлику
        let relative_json_file_path = full_json_file_path
            .strip_prefix(&arguments.json_files_directory)
            .wrap_err("Relative path fail")?;

        // Перегоняем в utf-8 строку
        let relative_path_str = relative_json_file_path
            .to_str()
            .ok_or_else(|| eyre::eyre!("Convert path to utf-8 string failed, path = {:?}", &relative_json_file_path))?;

        // Проверяем имя
        if !relative_path_str.ends_with("texture.json") && !relative_path_str.ends_with("_tex.json") {
            continue;
        }

        // Проверяем валидность
        let status = validate_json_file(&full_json_file_path)?;

        // Если файлик невалидный
        use ansi_term::Color::Blue;
        match status {
            ValidateResult::InvalidSize { image_name } => {
                // Путь к файлику картинки
                let image_path = {
                    // Путь к файлику в той же директории, что и json?
                    let same_dir_image_path = full_json_file_path.with_file_name(&image_name);
                    if same_dir_image_path.exists() {
                        same_dir_image_path
                    } else {
                        // В качестве параметров есть альтернативный путь к картинкам?
                        let alternative_dir_image_path = arguments
                            .alternative_texture_files_directory
                            .as_ref()
                            .map(|alternative_root| alternative_root.join(relative_json_file_path.with_file_name(image_name)));
                        // Проверяем доступноость
                        match alternative_dir_image_path {
                            Some(alternative_dir_image_path) if alternative_dir_image_path.exists() => alternative_dir_image_path,
                            _ => {
                                return Err(eyre::eyre!("Image is missing for json file: {:?}", full_json_file_path));
                            }
                        }
                    }
                };

                // Узнаем необходимый размер текстуры
                let texture_size = try_get_image_size(&image_path).wrap_err("Image size receive")?;

                // Нужно ли домножить размеры на 2
                let texture_size = if arguments.x2_texture_size {
                    texture_size * 2_u32
                } else {
                    texture_size
                };

                // Пишем сообщение с ошибкой в строку
                writeln!(
                    &mut err_lines,
                    "- {}: valid size = {{width: {}, height: {}}}",
                    Blue.paint(relative_path_str),
                    texture_size.width,
                    texture_size.height
                )?;
            }
            ValidateResult::NoImageName => {
                // Пишем сообщение с ошибкой в строку
                writeln!(&mut err_lines, "- {}: imagePath field is missing", Blue.paint(relative_path_str))?;
            }
            ValidateResult::Valid => {}
        }
    }

    // Если были найдены такие файлики
    if !err_lines.is_empty() {
        let root_path_str = arguments
            .json_files_directory
            .to_str()
            .wrap_err("Root path to utf-8 convert failed")?;

        use ansi_term::Color::Purple;
        use ansi_term::Color::Red;

        // TODO: выводить в stderr, но cmake как-то странно форматирует
        println!(
            "{} {}\n{}",
            Red.paint("Invalid dragonbones files found at folder:"),
            Purple.paint(root_path_str),
            err_lines
        );

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
