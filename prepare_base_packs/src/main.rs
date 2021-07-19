mod app_arguments;
mod helpers;
mod regex;
mod types;

use crate::{
    app_arguments::AppArguments,
    helpers::create_dir_for_file,
    types::{ConvertConfig, FoundEntry},
};
use eyre::{ContextCompat, WrapErr};
// use fallible_iterator::{FallibleIterator, FromFallibleIterator, IntoFallibleIterator};
// use itertools::Itertools;
use rayon::prelude::*;
use std::{
    convert::TryInto,
    fs::File,
    io::BufReader,
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
    for dir in arguments.source_directories.iter() {
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

#[instrument(level = "error", skip(packs_directory, valid_prefixes))]
fn get_pack_directories(packs_directory: &Path, valid_prefixes: &[impl AsRef<str> + Sync + Send]) -> Result<Vec<PathBuf>, eyre::Error> {
    // Сначала получим директории паков
    let mut packs_directories: Vec<PathBuf> = WalkDir::new(packs_directory)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .par_bridge()
        .map(|entry| -> Result<Option<PathBuf>, eyre::Error> {
            let _span = tracing::error_span!("Packs filter", full_path = tracing::field::Empty);
            let _span_guard = _span.enter();

            // Полный путь к директории
            let full_path = entry?.into_path();

            _span.record("full_path", &tracing::field::debug(&full_path));

            // Только директории
            if !full_path.is_dir() {
                return Ok(None);
            }

            // Относительный путь
            let relative_path = full_path.strip_prefix(packs_directory)?;

            // Получаем имя этой самой директории как первый компонент
            let dir_name_str = relative_path
                .components()
                .next()
                .ok_or_else(|| eyre::eyre!("Component is missing"))?
                .as_os_str()
                .to_str()
                .ok_or_else(|| eyre::eyre!("Component convert to string failed"))?;

            // Директория относится к префиксамвалидным?
            let is_found_prefix = valid_prefixes.iter().any(|prefix| dir_name_str.starts_with(prefix.as_ref()));

            if is_found_prefix {
                Ok(Some(packs_directory.join(dir_name_str)))
            } else {
                Ok(None)
            }
        })
        .filter_map(|entry| entry.transpose())
        .panic_fuse()
        .collect::<Result<Vec<PathBuf>, eyre::Error>>()?;

    // Выполним сортировку полученных директорий
    packs_directories.par_sort_unstable_by(PathBuf::cmp);

    Ok(packs_directories)
}

#[instrument(level = "error", skip(regexes, text))]
fn check_string_match(regexes: &[impl AsRef<fancy_regex::Regex>], text: &str) -> Result<bool, eyre::Error> {
    for re in regexes.iter() {
        if re.as_ref().is_match(text)? {
            return Ok(true);
        }
    }
    return Ok(false);
}

#[instrument(level = "error", skip(arguments, config))]
fn filter_found_path(
    arguments: &AppArguments,
    config: &ConvertConfig,
    root: &Path,
    path: PathBuf,
) -> Result<Option<FoundEntry>, eyre::Error> {
    // Работаем только с файликами
    if !path.is_file() {
        return Ok(None);
    }

    // Получаем относительный путь
    let relative_path = path.strip_prefix(&root).wrap_err("Invalid prefix")?;

    // Относительный путь начинается с res?
    if !relative_path.starts_with("res") {
        return Ok(None);
    }

    // Файл в папке, которую мы игнорируем?
    let parent_relative_path_str = relative_path.parent().and_then(|p| p.to_str());
    if let Some(parent_relative_path_str) = parent_relative_path_str {
        if check_string_match(&config.ignore_dirs, parent_relative_path_str)? {
            return Ok(None);
        }
    }

    // Файлик относится к игнорируемым?
    let relative_path_str = relative_path.to_str().ok_or_else(|| eyre::eyre!("To string convert failed"))?;
    if check_string_match(&config.ignore_files, relative_path_str)? {
        return Ok(None);
    }

    // Проверяем куда совать файлик
    let target_dir = if check_string_match(&config.forced_include_files_in_build, relative_path_str)? {
        // Файлик из разряда обязательных клиенту
        &arguments.target_client_res_directory
    } else if check_string_match(&config.exclude_files_from_build, relative_path_str)? {
        // Файлик из разряда для сервера
        &arguments.target_server_res_directory
    } else {
        // По-умолчанию попадает в клиент
        &arguments.target_client_res_directory
    };

    // Результат
    let full_target_path = target_dir.join(relative_path);
    return Ok(Some(FoundEntry {
        full_source_path: path,
        full_target_path,
    }));
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
    let config: ConvertConfig = {
        let file = File::open(&arguments.config_json).wrap_err("Config file open failed")?;
        let raw_conf = serde_json::from_reader::<_, ConvertConfig>(BufReader::new(file)).wrap_err("Parse config failed")?;
        raw_conf.try_into()?
    };
    debug!(?config, "Config");

    // Результаты
    let packs_directories = get_pack_directories(&arguments.packs_directory, &arguments.packs_directory_prefixes)?;
    debug!(?packs_directories, "Found pack's directories");

    // Обходим ПОСЛЕДОВАТЕЛЬНО (не параллельно) директории, так как нам важен порядок
    arguments
        .source_directories
        .iter()
        // Прицепляем директории из паков
        .chain(packs_directories.iter())
        // Обрабатываем последовательно каждую найденную директорию
        .try_for_each(|dir_root_path| -> Result<(), eyre::Error> {
            // Обходим список файлов директории
            WalkDir::new(dir_root_path)
                .into_iter()
                .par_bridge()
                // Фильтруем
                .map(|entry| -> Result<Option<FoundEntry>, eyre::Error> {
                    let path = entry
                        .ok()
                        .map(|entry| entry.into_path())
                        .wrap_err_with(|| eyre::eyre!("Invalid entry"))?;
                    filter_found_path(&arguments, &config, dir_root_path, path)
                })
                // Непосредственно копируем результаты
                .try_for_each(|result| -> Result<(), eyre::Error> {
                    if let Some(result) = result? {
                        // Создаем директорию если еще не было
                        create_dir_for_file(&result.full_target_path)?;
                        // Копируем файлик
                        std::fs::copy(result.full_source_path, result.full_target_path)?;
                    }
                    Ok(())
                })
        })?;

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
