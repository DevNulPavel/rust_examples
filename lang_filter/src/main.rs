mod app_arguments;
mod types;

use crate::{app_arguments::AppArguments, types::FilterConfig};
use eyre::WrapErr;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fmt::Write as FmtWrite,
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path
};
use structopt::StructOpt;
use tracing::{debug, instrument, Level};
use walkdir::WalkDir;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Настойка уровня логирования
fn setup_logging(arguments: &AppArguments) -> Result<(), eyre::Error> {
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
        .wrap_err("Tracing init failed")
}

/// Выполняем валидацию переданных аргументов приложения
fn validate_arguments(arguments: &AppArguments) -> Result<(), eyre::Error> {
    // Валидация параметров приложения
    eyre::ensure!(
        arguments.lang_files_folder.exists(),
        "Input lang files directory does not exist at path: {:?}",
        arguments.lang_files_folder
    );
    eyre::ensure!(
        arguments.lang_files_folder.is_dir(),
        "Input lang files directory must be directory: {:?}",
        arguments.lang_files_folder
    );
    eyre::ensure!(
        arguments.filter_config_path.exists(),
        "Filter config does not exist: {:?}",
        arguments.filter_config_path
    );
    eyre::ensure!(
        arguments.filter_config_path.is_file(),
        "Filter config must be file: {:?}",
        arguments.filter_config_path
    );

    Ok(())
}

#[instrument(level = "error", skip(config))]
fn filter_lang(path: &Path, config: &FilterConfig) -> Result<(), eyre::Error> {
    debug!("Process found file");

    // Структура данных в файлике
    #[derive(Debug, Deserialize)]
    struct DataStruct {
        dict: HashMap<String, String>,
    }

    // Исходные данные
    let source_data: DataStruct = {
        // Откроем файлик
        let file = File::open(path).wrap_err("Source file open failed")?;
        serde_json::from_reader(BufReader::new(file)).wrap_err("Sourse lang read")?
    };

    // Фильтруем в буффер в оперативке
    let mut result_key_values = String::new();
    for (key, value) in source_data.dict.into_iter() {
        let valid = config.allowed_keys_regex.iter().any(|regex| regex.is_match(&key));
        if valid {
            let value = value.replace("\"", "\\\"");
            write!(result_key_values, r#""{}":"{}","#, key, value)?;
        }
    }

    // Убираем последнюю запятую
    if !result_key_values.is_empty() {
        result_key_values.pop();
    }

    // Сразу откроем файлик для записи туда
    let mut writer = {
        // Откроем файлик
        let file = File::create(path).wrap_err("Result file open failed")?;
        BufWriter::new(file)
    };

    // Пишем в файлик
    writer.write_all(br#"{"dict":{"#).wrap_err("Result write err")?;
    writer.write_all(result_key_values.as_bytes()).wrap_err("Result write err")?;
    writer.write_all(b"}}").wrap_err("Result write err")?;

    Ok(())
}

#[cfg(feature = "multithreaded")]
#[instrument(level = "error", skip(config, arguments))]
fn process(arguments: AppArguments, config: FilterConfig) -> Result<(), eyre::Error> {
    use rayon::prelude::*;

    WalkDir::new(&arguments.lang_files_folder)
        .into_iter()
        .par_bridge()
        .try_for_each(|entry| -> Result<(), eyre::Error>{
            let entry = entry.wrap_err("Invalid walkdir entry")?;

            // Путь
            let path = entry.into_path();

            // Имя файлика
            let filename = match path.file_name().and_then(|name| name.to_str()) {
                Some(filename) => filename,
                None => return Ok(()),
            };

            // Подходящее имя файлика?
            if filename != "strings.json" {
                return Ok(());
            }

            // Файлик вообще?
            if !path.is_file() {
                return Ok(());
            }

            // Выполняем фильтрацию
            filter_lang(&path, &config)
        })
}

#[cfg(not(feature = "multithreaded"))]
#[instrument(level = "error", skip(config, arguments))]
fn process(arguments: AppArguments, config: FilterConfig) -> Result<(), eyre::Error> {
    // Однопоточный вариант
    let files_iter = WalkDir::new(&arguments.lang_files_folder);
    'cur_loop: for entry in files_iter {
        let entry = entry.wrap_err("Invalid walkdir entry")?;

        // Путь
        let path = entry.into_path();

        // Имя файлика
        let filename = match path.file_name().and_then(|name| name.to_str()) {
            Some(filename) => filename,
            None => continue 'cur_loop,
        };

        // Подходящее имя файлика?
        if filename != "strings.json" {
            continue 'cur_loop;
        }

        // Файлик вообще?
        if !path.is_file() {
            continue 'cur_loop;
        }

        // Выполняем фильтрацию
        filter_lang(&path, &config).wrap_err("Filtering")?;
    }
}

fn execute_app() -> Result<(), eyre::Error> {
    // Человекочитаемый вывод паники
    color_backtrace::install();

    // Настройка color eyre для ошибок
    color_eyre::install().wrap_err("Error setup failed")?;

    // Аргументы коммандной строки
    let arguments = app_arguments::AppArguments::from_args_safe().wrap_err("Arguments parsing")?;

    // Настройка логирования на основании количества флагов verbose
    setup_logging(&arguments).wrap_err("Logging setup")?;

    // Display arguments
    debug!(?arguments, "App arguments");

    // Валидация параметров приложения
    validate_arguments(&arguments).wrap_err("Arguments validate")?;

    // Распарсим конфиг
    let config: FilterConfig = {
        let file =
            File::open(&arguments.filter_config_path).wrap_err_with(|| format!("Open file {:?} failed", &arguments.filter_config_path))?;
        let config = serde_json::from_reader(BufReader::new(file)).wrap_err("Config parse failed")?;
        config
    };
    debug!(?config, "Config");

    // Непосредственно фильтрации
    process(arguments, config)
}

fn main() {
    // Запуск приложения
    if let Err(err) = execute_app() {
        // При ошибке не паникуем, а спокойно выводим сообщение и завершаем приложение с кодом ошибки
        eprint!("Error! Failed with: {:?}", err);
        std::process::exit(1);
    }
}
