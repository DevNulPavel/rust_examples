mod app_arguments;
mod helpers;
mod json;
mod types;

use crate::{
    app_arguments::AppArguments,
    types::{JsonInfo, JsonType},
};
use eyre::WrapErr;
use rayon::prelude::*;
use std::{
    fs::File,
    io::{Read, Write},
};
use structopt::StructOpt;
use tracing::{debug, instrument, Level};
use walkdir::WalkDir;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Настойка уровня логирования
fn setup_logging(arguments: &AppArguments) {
    use tracing_subscriber::prelude::*;

    // Настройка логирования на основании количества флагов verbose
    let level = match arguments.verbose {
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        4 => Level::TRACE,
        _ => {
            panic!("Verbose level must be in [0, 4] range");
        }
    };
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::from_level(level))
        .with(tracing_subscriber::filter::EnvFilter::new(env!("CARGO_PKG_NAME"))) // Логи только от текущего приложения, без библиотек
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_error::ErrorLayer::default()) // Для поддержки захватывания SpanTrace в eyre
        .try_init()
        .expect("Tracing init failed");
}

/// Выполняем валидацию переданных аргументов приложения
fn validate_arguments(arguments: &AppArguments) {
    // Валидация параметров приложения
    assert!(
        arguments.input_folder.exists(),
        "Input directory does not exist at path: {:?}",
        arguments.input_folder
    );
}

#[instrument(level = "error")]
fn minify_json(json_info: JsonInfo) -> Result<(), eyre::Error> {
    let mut src_file = File::open(&json_info.path)?;

    match json_info.json_type {
        JsonType::Encoded => {
            let mut src_data_buffer = Vec::new();
            src_file.read_to_end(&mut src_data_buffer)?;
            drop(src_file);

            src_data_buffer.iter_mut().for_each(|val| {
                *val = *val ^ 0xA5_u8;
            });

            let source_text = std::str::from_utf8(&src_data_buffer)?;

            let result_json_text = minifier::json::minify(&source_text);
            drop(src_data_buffer);

            let mut result_json_data = result_json_text.into_bytes();

            result_json_data.iter_mut().for_each(|val| {
                *val = *val ^ 0xA5_u8;
            });

            let mut res_file = File::create(&json_info.path)?;
            res_file.write_all(&result_json_data)?;
        }
        JsonType::Raw => {
            let mut src_data_buffer = String::new();
            src_file.read_to_string(&mut src_data_buffer)?;
            drop(src_file);

            let result_data_buffer = minifier::json::minify(&src_data_buffer);

            let mut res_file = File::create(&json_info.path)?;
            res_file.write_all(result_data_buffer.as_bytes())?;
        }
    }

    Ok(())
}

fn main() {
    // Человекочитаемый вывод паники
    color_backtrace::install();

    // Настройка color eyre для ошибок
    color_eyre::install().expect("Error setup failed");

    // Аргументы коммандной строки
    let arguments = app_arguments::AppArguments::from_args();

    // Настройка логирования на основании количества флагов verbose
    setup_logging(&arguments);

    // Display arguments
    debug!(?arguments, "App arguments");

    // Валидация параметров приложения
    validate_arguments(&arguments);

    WalkDir::new(&arguments.input_folder)
        // Параллельное итерирование
        .into_iter()
        // Параллелизация по потокам
        .par_bridge()
        // Только валидные папки и файлики
        .filter_map(|entry| entry.ok())
        // Конвертация в Path
        .map(|entry| entry.into_path())
        // Фильтруем только атласы
        .filter_map(|path| {
            // trace!(?path, "Check entry");

            let ext = match path.extension().and_then(|ext| ext.to_str()) {
                Some(ext) => ext,
                None => return None,
            };

            match ext {
                "json" => {
                    return Some(JsonInfo {
                        path,
                        json_type: JsonType::Raw,
                    });
                }
                "code" => {
                    let full_name = path.to_str().expect("Full path unwrap err");
                    if full_name.ends_with(".json.code") {
                        return Some(JsonInfo {
                            path,
                            json_type: JsonType::Encoded,
                        });
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
        // Непосредственно конвертация
        .for_each(|info| {
            debug!(?info, "Found entry");

            if let Err(err) = minify_json(info) {
                // При ошибке не паникуем, а спокойно выводим сообщение и завершаем приложение с кодом ошибки
                eprint!("Error! Failed with: {:?}", err);
                std::process::exit(1);
            }
        });
}
