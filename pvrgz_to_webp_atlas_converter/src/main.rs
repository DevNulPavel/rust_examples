mod app_arguments;
mod cache;
mod helpers;
mod json;
mod pvrgz;
mod types;

use crate::{
    app_arguments::AppArguments,
    cache::CacheInfo,
    json::correct_file_name_in_json,
    pvrgz::pvrgz_to_webp,
    types::{AtlasInfo, UtilsPathes},
};
use eyre::WrapErr;
use rayon::prelude::*;
use std::fs::remove_file;
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
        arguments.atlases_images_directory.exists(),
        "Atlasses directory does not exist at path: {:?}",
        arguments.atlases_images_directory
    );
    assert!(arguments.target_webp_quality <= 100, "Target webp quality must be from 0 to 100");
    if let Some(alternative_atlases_json_directory) = arguments.alternative_atlases_json_directory.as_ref() {
        assert!(
            alternative_atlases_json_directory.exists(),
            "Atlasses alternative json directory does not exist at path: {:?}",
            alternative_atlases_json_directory
        );
    }
}

#[instrument(level = "error", skip(cache_info, utils_pathes))]
fn convert_pvrgz_atlas_to_webp(
    cache_info: &CacheInfo,
    utils_pathes: &UtilsPathes,
    target_webp_quality: u8,
    info: AtlasInfo,
) -> Result<(), eyre::Error> {
    // Из .pvrgz в .webp
    pvrgz_to_webp(cache_info, utils_pathes, target_webp_quality, &info.pvrgz_path).wrap_err("Pvrgz to webp convert")?;

    // Удаляем старый .pvrgz
    remove_file(&info.pvrgz_path).wrap_err("Pvrgz delete failed")?;

    // Правим содержимое .json файлика, прописывая туда .новое имя файла
    correct_file_name_in_json(cache_info, &info.json_path).wrap_err("Json fix failed")?;

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

    WalkDir::new(&arguments.atlases_images_directory)
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

            // Рядом с ним есть такой же .json?
            let same_folder_atlas_json_file = path.with_extension("json");
            if same_folder_atlas_json_file.exists() {
                // Возвращаем
                return Some(AtlasInfo {
                    pvrgz_path: path,
                    json_path: same_folder_atlas_json_file,
                });
            }

            // Может быть есть .json в отдельной директории?
            if let Some(alternative_atlases_json_directory) = arguments.alternative_atlases_json_directory.as_ref() {
                let relative_json_atlas_path = same_folder_atlas_json_file
                    .strip_prefix(&arguments.atlases_images_directory)
                    .expect("Images json prefix strip failed");
                let external_folder_atlas_json_file = alternative_atlases_json_directory.join(relative_json_atlas_path);
                if external_folder_atlas_json_file.exists() {
                    // Возвращаем
                    return Some(AtlasInfo {
                        pvrgz_path: path,
                        json_path: external_folder_atlas_json_file,
                    });
                }
            }

            None
        })
        // Непосредственно конвертация
        .for_each(|info| {
            debug!(?info, "Found atlas entry");

            if let Err(err) = convert_pvrgz_atlas_to_webp(&cache_info, &utils_pathes, arguments.target_webp_quality, info) {
                // При ошибке не паникуем, а спокойно выводим сообщение и завершаем приложение с кодом ошибки
                eprint!("Error! Failed with: {:?}", err);
                std::process::exit(1);
            }
        });
}
