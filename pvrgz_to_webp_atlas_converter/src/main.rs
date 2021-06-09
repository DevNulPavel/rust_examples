mod app_arguments;
mod helpers;
mod json_fix;
mod pvrgz_to_webp;
mod types;

use crate::{
    app_arguments::AppArguments,
    helpers::{create_dir_for_file, replace_root_on_path},
    json_fix::correct_file_name_in_json,
    pvrgz_to_webp::pvrgz_to_webp,
    types::{AtlasInfo, ConvertEntry, UtilsPathes},
};
use eyre::WrapErr;
use rayon::prelude::*;
use std::{
    fs::{create_dir_all, File},
    io::Read,
    path::Path,
};
use structopt::StructOpt;
use tracing::{debug, trace, instrument, Level};
use walkdir::WalkDir;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Настойка уровня логирования
/*fn setup_logging(arguments: &AppArguments) {
    // Настройка логирования на основании количества флагов verbose
    let level = match arguments.verbose {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        4 => log::LevelFilter::Trace,
        _ => {
            panic!("Verbose level must be in [0, 4] range");
        }
    };
    pretty_env_logger::formatted_builder()
        .filter_level(level)
        .try_init()
        .expect("Logger init failed");
}*/

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
        .with(tracing_subscriber::filter::EnvFilter::new(env!("CARGO_PKG_NAME"))) // Фильтруем лишь логи текущего приложения
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(tracing_error::ErrorLayer::default()) // Для поддержки захватывания SpanTrace в eyre
        .try_init()
        .expect("Tracing init failed");
}

/// Выполняем валидацию переданных аргументов приложения
fn validate_arguments(arguments: &AppArguments) {
    // Валидация параметров приложения
    // Sources dir
    assert!(
        arguments.source_directory.exists(),
        "Atlasses directory does not exist at path: {:?}",
        arguments.source_directory
    );
    // Web quality
    assert!(
        arguments.target_webp_quality <= 100 && arguments.target_webp_quality > 0,
        "Target webp quality must be in [1,100] range"
    );
}

#[instrument(level = "error")]
fn get_md5_for_file(path: &Path) -> Result<md5::Digest, eyre::Error> {
    let mut md5 = md5::Context::new();
    let mut file = File::open(path).wrap_err("File open")?;
    let mut buffer = [0_u8; 4096];
    loop {
        let read_count = file.read(&mut buffer)?;
        if read_count == 0 {
            break;
        }
        md5.consume(&buffer[0..read_count]);
    }
    Ok(md5.compute())
}

#[instrument(level = "error", skip(db))]
fn copy_file_if_needed(db: &sled::Db, source_root: &Path, target_root: &Path, absolute_file_path: &Path) -> Result<(), eyre::Error> {
    // Вычисляем хэш от файлика исходного
    let source_md5 = format!("{:x}", get_md5_for_file(absolute_file_path).wrap_err("Source md5 calculate")?);

    // Относительный путь к файлику
    let relative_path = absolute_file_path.strip_prefix(source_root).wrap_err("Relative source path")?;
    let relative_path_str = relative_path.to_str().ok_or_else(|| eyre::eyre!("Relative source to str"))?;

    // Ищем хэш в базе для данного файлика
    let db_hash = db.get(relative_path_str).wrap_err("Source hash read")?;

    // Старый хэш был и он совпадает?
    if let Some(prev_hash) = db_hash {
        if prev_hash == source_md5 {
            debug!(?absolute_file_path, "Hash is valid, do not copy");
            return Ok(());
        }
    }

    // Копируем файлик и сохраняем хэш
    let target_file_path = target_root.join(relative_path);
    debug!(?absolute_file_path, ?target_file_path, "Copy file");
    create_dir_for_file(&target_file_path).wrap_err_with(|| format!("target dir create failed: {:?}", &target_file_path))?;
    std::fs::copy(&absolute_file_path, target_file_path).wrap_err("File copy")?;
    db.insert(relative_path_str, source_md5.as_str()).wrap_err("Db hash save")?;

    Ok(())
}

#[derive(Debug)]
struct AtlasHashes<'a> {
    json_key: &'a str,
    json_hash: String,
    pvrgz_key: &'a str,
    pvrgz_hash: String,
}
impl<'a> AtlasHashes<'a> {
    fn save_in_db(&self, db: &sled::Db) -> Result<(), eyre::Error> {
        db.insert(self.pvrgz_key, self.pvrgz_hash.as_str()).wrap_err("Db hash save")?;
        db.insert(self.json_key, self.json_hash.as_str()).wrap_err("Db hash save")?;
        Ok(())
    }
}

/// Возвращает новые хеши, если что-то поменялось
#[instrument(level = "error", skip(db))]
fn check_atlas_hashed<'a>(db: &sled::Db, source_root: &Path, atlas_info: &'a AtlasInfo) -> Result<Option<AtlasHashes<'a>>, eyre::Error> {
    let source_pvrgz_md5 = format!("{:x}", get_md5_for_file(&atlas_info.pvrgz_path).wrap_err("Source pvrgz md5 calculate")?);
    let source_json_md5 = format!("{:x}", get_md5_for_file(&atlas_info.json_path).wrap_err("Source json md5 calculate")?);
    let relative_pvrgz_path_str = atlas_info
        .pvrgz_path
        .strip_prefix(source_root)
        .wrap_err("Relative pvrgz source path")?
        .to_str()
        .ok_or_else(|| eyre::eyre!("Non ascii pvrgz path"))?;
    let relative_json_path_str = atlas_info
        .json_path
        .strip_prefix(source_root)
        .wrap_err("Relative json source path")?
        .to_str()
        .ok_or_else(|| eyre::eyre!("Non ascii json path"))?;

    // Получаем из базы хеши и проверяем, что совпадает
    if let (Some(prev_pvrgz_hash), Some(prev_json_hash)) = (db.get(relative_pvrgz_path_str)?, db.get(relative_json_path_str)?) {
        // Если совпадет, значит не нужны новые хеши
        if prev_pvrgz_hash == source_pvrgz_md5 || prev_json_hash == source_json_md5 {
            return Ok(None);
        }
    }

    // Если у нас в базе не было хешей каких-то, тогда тоже заново стартуем
    return Ok(Some(AtlasHashes {
        pvrgz_key: relative_pvrgz_path_str,
        pvrgz_hash: source_pvrgz_md5,
        json_key: relative_json_path_str,
        json_hash: source_json_md5,
    }));
}

#[instrument(level = "error", skip(db, arguments, utils_pathes))]
fn process_found_entry(db: &sled::Db, utils_pathes: &UtilsPathes, arguments: &AppArguments, entry: ConvertEntry) -> Result<(), eyre::Error> {
    match entry {
        ConvertEntry::Atlas(atlas_info) => {
            trace!(?atlas_info, "Found atlas entry");

            // Размер файла слишком мелкий? Тогда просто копируем файлик и json
            let meta = std::fs::metadata(&atlas_info.pvrgz_path).expect("File metadata read failed");
            if meta.len() >= arguments.minimum_pvrgz_size {
                // Читаем хэши исходных файлов и проверяем их c тем, что в базе
                // Если что-то изменилось, значит конвертируем
                let new_hashes = match check_atlas_hashed(db, &arguments.source_directory, &atlas_info).wrap_err("Atlas hashes check")? {
                    Some(new_hashes) => {
                        debug!(pvrgz = ?atlas_info.pvrgz_path, json = ?atlas_info.json_path, ?new_hashes, "Hashes is not same, convert it");
                        new_hashes
                    },
                    None => {
                        debug!(pvrgz = ?atlas_info.pvrgz_path, json = ?atlas_info.json_path, "Hash is valid, do not convert it");
                        return Ok(());
                    }
                };

                // Пути новые для .webp
                let target_webp_path =
                    replace_root_on_path(&atlas_info.pvrgz_path, &arguments.source_directory, &arguments.target_directory)?.with_extension("webp");

                // Создание директории
                create_dir_for_file(&target_webp_path).wrap_err_with(|| format!("target dir create failed: {:?}", &target_webp_path))?;

                // Из .pvrgz в .webp
                pvrgz_to_webp(utils_pathes, arguments.target_webp_quality, &atlas_info.pvrgz_path, &target_webp_path)
                    .wrap_err("Pvrgz to webp convert")?;

                // Новая директория
                let target_json_path = replace_root_on_path(&atlas_info.json_path, &arguments.source_directory, &arguments.target_directory)?;

                // Копия json файлика
                std::fs::copy(&atlas_info.json_path, &target_json_path).wrap_err("Json copy")?;

                // Правим содержимое .json файлика, прописывая туда .новое имя файла
                correct_file_name_in_json(&target_json_path).wrap_err("Json fix failed")?;

                // Если все успешно, тогда обновляем хеши в базе
                new_hashes.save_in_db(db)?;
            } else {
                // Просто выполняем копирование с проверкой обоих файликов
                copy_file_if_needed(db, &arguments.source_directory, &arguments.target_directory, &atlas_info.pvrgz_path).wrap_err("File copy")?;
                copy_file_if_needed(db, &arguments.source_directory, &arguments.target_directory, &atlas_info.json_path).wrap_err("File copy")?;
            }
        }
        ConvertEntry::OtherFile(source_path) => {
            // Обычный файлик просто копируем
            copy_file_if_needed(db, &arguments.source_directory, &arguments.target_directory, &source_path).wrap_err("File copy")?;
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

    // Находим пути к бинарникам для конвертации
    let utils_pathes = UtilsPathes {
        pvr_tex_tool: which::which("PVRTexToolCLI").expect("PVRTexTool application not found"),
        cwebp: which::which("cwebp").expect("PVRTexTool application not found"),
    };
    debug!(?utils_pathes, "Utils pathes");

    // Есть ли директория с результатами?
    create_dir_all(&arguments.target_directory).expect("Target directory create failed");

    // Открываем базу данных с хэшами
    let db = sled::open(&arguments.hashes_db_path).expect("Hashes database open failed");

    WalkDir::new(&arguments.source_directory)
        // Параллельное итерирование
        .into_iter()
        // Параллелизация по потокам
        .par_bridge()
        // Только валидные папки и файлики
        .filter_map(|entry| entry.ok())
        // Конвертация в Path
        .map(|entry| entry.into_path())
        // Только файлы
        .filter(|path| path.is_file())
        // Обработка каждого файлика
        .filter_map(|path| {
            // Смотрим расширение файлика
            match path.extension().and_then(|ext| ext.to_str()) {
                // Это файлик .pvrgz?
                Some("pvrgz") => {
                    // Рядом с ним есть такой же .json?
                    let same_folder_atlas_json_file = path.with_extension("json");
                    if same_folder_atlas_json_file.exists() {
                        // Возвращаем данные по атласу
                        return Some(ConvertEntry::Atlas(AtlasInfo {
                            pvrgz_path: path,
                            json_path: same_folder_atlas_json_file,
                        }));
                    }
                }
                // Это .json файлик?
                Some("json") => {
                    // Рядом с ним есть такой же .pvrgz?
                    let same_folder_atlas_pvrgz_file = path.with_extension("pvrgz");
                    if same_folder_atlas_pvrgz_file.exists() {
                        // Тогда пропускаем файлик
                        return None;
                    }
                }
                // Просто любой другой файлик - возвращаем путь просто
                _ => {}
            }

            // Просто любой другой файлик
            return Some(ConvertEntry::OtherFile(path));
        })
        .for_each(|entry| {
            // При ошибке не паникуем, а спокойно выводим сообщение и завершаем приложение с кодом ошибки
            if let Err(err) = process_found_entry(&db, &utils_pathes, &arguments, entry) {
                eprint!("Error! Failed with: {:?}", err);
                std::process::exit(1);
            }
        });

    db.flush().expect("Database flush failed");
}
