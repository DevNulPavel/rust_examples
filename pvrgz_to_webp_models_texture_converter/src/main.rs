mod app_arguments;
mod cache;
mod helpers;
mod pvrgz;
mod types;

use crate::{app_arguments::AppArguments, cache::CacheInfo, pvrgz::pvrgz_to_webp, types::UtilsPathes};
use eyre::WrapErr;
use rayon::prelude::*;
use std::{fs::remove_file, path::PathBuf};
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
    arguments.pvrgz_directories.iter().for_each(|dir| {
        assert!(dir.exists(), "Images directory does not exist at path: {:?}", dir);
    });
    assert!(arguments.target_webp_quality <= 100, "Target webp quality must be from 0 to 100");
}

#[instrument(level = "error", skip(cache_info, utils_pathes))]
fn convert_pvrgz_to_webp(
    cache_info: &CacheInfo,
    utils_pathes: &UtilsPathes,
    target_webp_quality: u8,
    pvrgz_path: PathBuf,
) -> Result<(), eyre::Error> {
    // Из .pvrgz в .webp
    pvrgz_to_webp(cache_info, utils_pathes, target_webp_quality, &pvrgz_path).wrap_err("Pvrgz to webp convert")?;

    // Удаляем старый .pvrgz
    remove_file(&pvrgz_path).wrap_err("Pvrgz delete failed")?;

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

    // Находим пути к бинарникам для конвертации
    let utils_pathes = UtilsPathes {
        pvr_tex_tool: which::which("PVRTexToolCLI").expect("PVRTexTool application not found"),
        cwebp: which::which("cwebp").expect("PVRTexTool application not found"),
    };
    debug!(?utils_pathes, "Utils pathes");

    // Создаем директории для кеша и открываем базу для хешей
    let cache_info = CacheInfo::open(&arguments.cache_path);

    // Открываем файлик, в котором будем заменять имена старых файлов на новые
    // TODO: ???

    // Идем по всем нашим директориям
    arguments
        .pvrgz_directories
        // Параллельный незаимствующий итератор
        .par_iter()
        // Для полного параллелизма между итераторами по директориям используем flat_map + par_bridge
        .flat_map(|dir| WalkDir::new(&dir).into_iter().par_bridge())
        // Только валидные папки и файлики
        .filter_map(|entry| entry.ok())
        // Конвертация в Path
        .map(|entry| entry.into_path())
        // Фильтруем только .pvrgz
        .filter_map(|path| {
            // Только файлики
            if !path.is_file() {
                return None;
            }

            // Это файлик .pvrgz?
            match path.extension().and_then(|ext| ext.to_str()) {
                Some("pvrgz") => {}
                _ => return None,
            }

            // Размер файла слишком мелкий? Тогда не трогаем - это может быть заглушка, либо это бессмысленно
            let meta = std::fs::metadata(&path).expect("File metadata read failed");
            if meta.len() < arguments.minimum_pvrgz_size {
                return None;
            }

            Some(path)
        })
        // Непосредственно конвертация
        .for_each(|path| {
            debug!(?path, "Found pvrgz");

            if let Err(err) = convert_pvrgz_to_webp(&cache_info, &utils_pathes, arguments.target_webp_quality, path) {
                // При ошибке не паникуем, а спокойно выводим сообщение и завершаем приложение с кодом ошибки
                eprint!("Error! Failed with: {:?}", err);
                std::process::exit(1);
            }
        });

    // После конвертации сохраняем новое содержимое .json файлика
}
