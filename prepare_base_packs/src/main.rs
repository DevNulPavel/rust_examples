mod app_arguments;
mod helpers;
mod types;

use crate::{app_arguments::AppArguments, types::UtilsPathes};
use eyre::WrapErr;
use itertools::Itertools;
use rayon::prelude::*;
use serde::Deserialize;
use std::{
    fs::{remove_file, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::RwLock,
};
use structopt::StructOpt;
use tracing::{debug, instrument, Level};
use walkdir::WalkDir;
// use fallible_iterator::{FallibleIterator, IntoFallibleIterator, FromFallibleIterator};

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
    for dir in arguments.other_source_directories.iter() {
        eyre::ensure!(dir.is_dir(), "Source directory must be directory at path: {:?}", dir);
        eyre::ensure!(dir.exists(), "Source directory does not exist at path: {:?}", dir);
    }

    eyre::ensure!(
        arguments.packs_directory.exists(),
        "Packs directory does not exist at path: {:?}",
        arguments.packs_directory
    );
    eyre::ensure!(
        arguments.packs_directory.is_dir(),
        "Packs directory must be a dir: {:?}",
        arguments.packs_directory
    );

    eyre::ensure!(
        arguments.config_json.exists(),
        "Config does not exist at path: {:?}",
        arguments.config_json
    );
    eyre::ensure!(
        arguments.config_json.is_file(),
        "Config must be file at path: {:?}",
        arguments.config_json
    );

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub ignore_dirs: Vec<String>,
    pub ignore_files: Vec<String>,
    pub exclude_files_from_build: Vec<String>,
    pub forced_include_files_in_build: Vec<String>,
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

    // Открываем файлик конфига и парсим данные из него
    let config = {
        let file = File::open(&arguments.config_json).wrap_err("Config file open failed")?;
        serde_json::from_reader::<_, Config>(file).wrap_err("Parse config failed")?
    };

    struct FoundEntry<'a> {
        root: &'a Path,
        full_path: PathBuf,
    }

    // Сначала идем по директории с паками
    WalkDir::new(&arguments.packs_directory)
        .into_iter()
        .par_bridge()
        .map(|entry| -> Result<Option<FoundEntry>, eyre::Error> {
            let full_path = entry?.into_path();
            let relative_path_str = full_path
                .strip_prefix(&arguments.packs_directory)?
                .to_str()
                .ok_or_else(|| eyre::eyre!("Path to string convert failed"))?;

            let valid = arguments
                .packs_directory_prefixes
                .iter()
                .any(|prefix| relative_path_str.starts_with(prefix));

            if valid {
                Ok(Some(FoundEntry {
                    full_path,
                    root: &arguments.packs_directory,
                }))
            } else {
                Ok(None)
            }
        })
        .filter_map(|entry| entry.transpose())
        .try_for_each(|entry| {
            Ok(())
        });

    arguments.other_source_directories.iter().try_for_each(|dir| {
        WalkDir::new(dir)
            .into_iter()
            .par_bridge()
            .map(move |entry| -> Result<FoundEntry, eyre::Error> {
                let full_path = entry?.into_path();
                Ok(FoundEntry { full_path, root: &dir })
            });

        Ok(())
    });

    // std::iter::once(packs_iter).chain(other_source_iterators);

    // // Идем по всем нашим директориям
    // arguments
    //     .source_directories
    //     // Параллельный незаимствующий итератор
    //     .par_iter()
    //     // Для полного параллелизма между итераторами по директориям используем flat_map + par_bridge
    //     .flat_map(|dir| std::iter::repeat(dir).zip(WalkDir::new(&dir).into_iter()).par_bridge())
    //     // Только валидные папки и файлики
    //     .filter_map(|(root, entry)| (root, entry.expect("Entry path err").into_path()))
    //     // Если кто-то запаниковал, тогда останавливаем работу остальных потоков
    //     .panic_fuse()
    //     // Непосредственно конвертация
    //     .for_each(|(root, path)| {

    //     });

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
