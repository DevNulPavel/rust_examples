mod app_arguments;
mod cache;
mod helpers;
mod pvrgz;
mod types;

use crate::{app_arguments::AppArguments, cache::CacheInfo, pvrgz::pvrgz_to_webp, types::UtilsPathes};
use eyre::WrapErr;
use rayon::prelude::*;
use std::{
    fs::{remove_file, File},
    io::{Read, Write, BufReader},
    path::{Path, PathBuf},
    sync::RwLock,
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
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        3 => Level::TRACE,
        _ => {
            panic!("Verbose level must be in [0, 3] range");
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
    if let Some(ignore_path) = &arguments.ignore_config_path {
        assert!(ignore_path.exists(), "Ignore config json file does not exist: {:?}", ignore_path);
        assert!(ignore_path.is_file(), "Ignore config must be file: {:?}", ignore_path);
    }
    assert!(arguments.target_webp_quality <= 100, "Target webp quality must be from 0 to 100");
}

#[instrument(level = "error", skip(models_json_string, path_root, pvrgz_path))]
fn replace_pvrgz_to_webp_path(models_json_string: &RwLock<String>, path_root: &Path, pvrgz_path: &Path) -> Result<(), eyre::Error> {
    // Относительный путь
    let old_relative_path = pvrgz_path.strip_prefix(path_root).wrap_err("Pvrgz path root strip")?;
    let old_relative_path_str = old_relative_path.to_str().ok_or_else(|| eyre::eyre!("Old path to string failed"))?;
    let new_relative_path = old_relative_path.with_extension("webp");
    let new_relative_path_str = new_relative_path.to_str().ok_or_else(|| eyre::eyre!("New path to string failed"))?;

    // Получаем блокировку на строке
    let mut data_guard = match models_json_string.write() {
        Ok(data_guard) => data_guard,
        Err(err) => {
            return Err(eyre::eyre!("Mutex lock error: {}", err));
        }
    };

    // Обновляем строку на новое значение пути
    // TODO: Очень много переаллокаций, может быть использовать предаллоцированную арену?
    // bumpalo? slab?
    *data_guard = data_guard.replace(old_relative_path_str, new_relative_path_str);

    Ok(())
}

#[instrument(level = "error", skip(cache_info, utils_pathes, arguments, models_json_string))]
fn process_found_path(
    cache_info: &CacheInfo,
    utils_pathes: &UtilsPathes,
    arguments: &AppArguments,
    models_json_string: &RwLock<String>,
    path_root: &Path,
    pvrgz_path: PathBuf,
) -> Result<(), eyre::Error> {
    // Из .pvrgz в .webp
    pvrgz_to_webp(cache_info, utils_pathes, arguments.target_webp_quality, &pvrgz_path).wrap_err("Pvrgz to webp convert")?;

    // Удаляем старый .pvrgz
    remove_file(&pvrgz_path).wrap_err("Pvrgz delete failed")?;

    // Заменяем старый путь в json на новый c новым расширением
    replace_pvrgz_to_webp_path(models_json_string, path_root, &pvrgz_path).wrap_err("Result json replace")?;

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

    // Читаем список для игнорирования из файлика если надо
    let ignore_list = arguments.ignore_config_path.as_ref().map(|path| {
        let file = File::open(path).expect("Ignore file open failed");
        serde_json::from_reader::<_, Vec<String>>(BufReader::new(file)).expect("Ignore list parse failed")
    });

    // Создаем директории для кеша и открываем базу для хешей
    let cache_info = CacheInfo::open(&arguments.cache_path);

    // Открываем файлик, в котором будем заменять имена старых файлов на новые
    let json_file_data = {
        let mut data: String = String::new();
        File::open(&arguments.models_config_json)
            .expect("Json file open error")
            .read_to_string(&mut data)
            .expect("File data read failed");

        // Завернем данные в Mutex для возможности изменять из разных потоков
        RwLock::new(data)
    };

    // Идем по всем нашим директориям
    arguments
        .pvrgz_directories
        // Параллельный незаимствующий итератор
        .par_iter()
        // Для полного параллелизма между итераторами по директориям используем flat_map + par_bridge
        .flat_map(|dir| std::iter::repeat(dir).zip(WalkDir::new(&dir).into_iter()).par_bridge())
        // Только валидные папки и файлики
        .map(|(root, entry)| (root, entry.expect("Entry path err").into_path()))
        // Фильтруем только .pvrgz
        .filter(|(root, path)| {
            // Только файлики
            if !path.is_file() {
                return false;
            }

            // Это файлик .pvrgz?
            match path.extension().and_then(|ext| ext.to_str()) {
                Some("pvrgz") => {}
                _ => return false,
            }

            // Размер файла слишком мелкий? Тогда не трогаем - это может быть заглушка, либо это бессмысленно
            let meta = std::fs::metadata(&path).expect("File metadata read failed");
            if meta.len() < arguments.minimum_pvrgz_size {
                return false;
            }

            // Относительный путь к файлу
            // TODO: Повторно вычисляется ниже, не дублировать?
            let relative_path_str = path
                .strip_prefix(root)
                .expect("Root strip error")
                .to_str()
                .expect("Path to str err");

            // В списке игнора?
            if let Some(ignore_list) = ignore_list.as_ref() {
                let contains_ignore = ignore_list.into_iter().any(|item| relative_path_str.starts_with(item));
                if contains_ignore {
                    return false;
                }
            }

            // Проверим сначала, что данная текстура у нас встречается вообще в файлике json
            {
                // Получаем блокировку на строке
                let data_guard = json_file_data.read().expect("Mutex lock failed");

                // Если не нашли, выходим с выводом предупреждения
                if data_guard.find(relative_path_str).is_none() {
                    // warn!(path = relative_path_str, "File not found in models.json");
                    return false;
                }
            }

            true
        })
        // Если кто-то запаниковал, тогда останавливаем работу остальных потоков
        .panic_fuse()
        // Непосредственно конвертация
        .for_each(|(root, path)| {
            debug!(?path, "Found pvrgz");

            if let Err(err) = process_found_path(&cache_info, &utils_pathes, &arguments, &json_file_data, root, path) {
                // При ошибке не паникуем, а спокойно выводим сообщение и завершаем приложение с кодом ошибки
                eprint!("Error! Failed with: {:?}", err);
                std::process::exit(1);
            }
        });

    // После конвертации сохраняем новое содержимое .json файлика
    {
        let mut file = File::create(arguments.models_config_json).expect("Json file open failed");
        let data_guard = json_file_data.read().expect("Mutex lock failed");
        file.write_all(data_guard.as_bytes()).expect("Json data save failed");
    }
}
