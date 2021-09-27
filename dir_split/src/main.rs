mod app_arguments;
mod cache;
mod helpers;

use crate::{
    app_arguments::{AppArguments, CompressionArg},
    cache::{save_in_cache, search_in_cache, CacheResult},
    helpers::create_dir_for_file,
};
use eyre::{ContextCompat, WrapErr};
use log::{debug, LevelFilter};
use rayon::prelude::*;
use std::{
    fs::{create_dir_all, remove_dir_all, rename as fs_rename, File},
    io::{copy as io_copy, BufReader},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use structopt::StructOpt;
use walkdir::WalkDir;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Настойка уровня логирования
fn setup_logging(arguments: &AppArguments) -> Result<(), eyre::Error> {
    // TODO: Фикс логов
    // Настройка логирования на основании количества флагов verbose
    let level = match arguments.verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        4 => LevelFilter::Trace,
        _ => {
            return Err(eyre::eyre!("Verbose level must be in [0, 4] range"));
        }
    };
    pretty_env_logger::formatted_timed_builder().filter_level(level).try_init()?;

    Ok(())
}

//////////////////////////////////////////////////////////////////////////////////////////////////////

/// Выполняем валидацию переданных аргументов приложения
fn validate_arguments(arguments: &AppArguments) -> Result<(), eyre::Error> {
    eyre::ensure!(arguments.resuld_dirs_count > 0, "Result dirs count must be greater than 0");

    eyre::ensure!(arguments.source_dirs_root.exists(), "Source root directory does not exist");
    eyre::ensure!(arguments.source_dirs_root.is_dir(), "Source root directory is not directory");

    eyre::ensure!(!arguments.source_dirs.is_empty(), "Source directories count must be greater than 0");
    for path in arguments.source_dirs.as_slice() {
        eyre::ensure!(path.is_relative(), "Source directory must be relative: {:?}", path);

        // TODO: Закешировать результаты в аргументах, либо как-то можно это сделать при парсинге даже?
        let full_path = arguments.source_dirs_root.join(path);
        eyre::ensure!(full_path.exists(), "Source directory does not exist: {:?}", path);
        eyre::ensure!(full_path.is_dir(), "Source directory must be dir: {:?}", path);
    }
    match arguments.compression_type {
        CompressionArg::Brotli | CompressionArg::Gzip => {
            eyre::ensure!(
                arguments.compression_level > 0 && arguments.compression_level < 12,
                "Compression level must be in range [1; 11]"
            );
        }
        CompressionArg::None => {}
    }

    Ok(())
}

//////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct Bucked {
    current_size: AtomicU64,
    result_root_path: PathBuf,
}
impl Bucked {
    fn new(result_root_path: PathBuf) -> Result<Bucked, eyre::Error> {
        // Создадим директорию для результата
        create_dir_all(&result_root_path).wrap_err_with(|| format!("Target dir create failed: {:?}", &result_root_path))?;

        Ok(Bucked {
            current_size: AtomicU64::new(0),
            result_root_path,
        })
    }

    fn put_file(&self, src_root: &Path, src_file_path: &Path, src_file_size: u64) -> Result<(), eyre::Error> {
        // Проверим, что параметры валидные
        eyre::ensure!(
            src_file_path.starts_with(src_root),
            "Source path {:?} must starts with source root {:?}",
            src_file_path,
            src_root
        );

        // Получим путь относительно корневой директории исходной
        let relative_src_path = src_file_path
            .strip_prefix(src_root)
            .wrap_err_with(|| format!("Source path {:?} must starts with source root {:?}", src_file_path, src_root))?;

        // Путь в конечную директорию
        let result_full_path = self.result_root_path.join(relative_src_path);

        // Директория конечная
        create_dir_for_file(&result_full_path).wrap_err_with(|| format!("Result dir create failed: {:?}", result_full_path))?;

        // Перемещаем файлик
        fs_rename(&src_file_path, &result_full_path)
            .wrap_err_with(|| format!("File copy failed: {:?} -> {:?}", src_file_path, result_full_path))?;

        // Копируем
        // fs_copy(&src_file_path, &result_full_path)
        //     .wrap_err_with(|| format!("File copy failed: {:?} -> {:?}", src_file_path, result_full_path))?;

        // Добавляем к конечному размеру размер результата, размер может быть после компрессии
        self.current_size.fetch_add(src_file_size, Ordering::SeqCst);

        Ok(())
    }
    fn current_size(&self) -> u64 {
        self.current_size.load(Ordering::SeqCst)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////

fn detect_result_file_size(
    compression_type: &CompressionArg,
    compression_level: u8,
    file_path: &Path,
    cache: &Option<sled::Db>,
) -> Result<u64, eyre::Error> {
    let mut file = File::open(&file_path).wrap_err_with(|| format!("Source file open failed: {:?}", file_path))?;

    let size = match compression_type {
        app_arguments::CompressionArg::Brotli => {
            // Ищем в кеше сначала
            let cache_save_key = match search_in_cache(file_path, &mut file, compression_type, compression_level, cache)? {
                CacheResult::Found { size } => return Ok(size),
                CacheResult::NotFound { cache_save_key } => Some(cache_save_key),
                CacheResult::NoCache => None,
            };

            // Компрессия файлика
            let mut reader = BufReader::new(file);
            let params = brotli::enc::BrotliEncoderParams {
                quality: compression_level as i32,
                ..Default::default()
            };
            let mut void_writer = std::io::sink();
            let size = brotli::BrotliCompress(&mut reader, &mut void_writer, &params).wrap_err("Brotli size analyze failed")? as u64;

            // Сохраним в кеш
            save_in_cache(&cache_save_key, size, cache)?;

            size
        }
        app_arguments::CompressionArg::Gzip => {
            // Ищем в кеше сначала
            let cache_save_key = match search_in_cache(file_path, &mut file, compression_type, compression_level, cache)? {
                CacheResult::Found { size } => return Ok(size),
                CacheResult::NotFound { cache_save_key } => Some(cache_save_key),
                CacheResult::NoCache => None,
            };

            // Компрессия файлика
            let mut reader = BufReader::new(file);
            let void_writer = std::io::sink();
            let level = match compression_level {
                0 => {
                    eyre::bail!("Zero compression unsupported");
                }
                1 | 2 | 3 | 4 | 5 => flate2::Compression::fast(),
                6 | 7 | 8 | 9 | 10 | 11 => flate2::Compression::best(),
                _ => flate2::Compression::best(),
            };
            let mut gz_writer = flate2::write::GzEncoder::new(void_writer, level);
            let size = io_copy(&mut reader, &mut gz_writer).wrap_err_with(|| format!("Gzip compression failed: {:?}", file_path))? as u64;

            // Сохраним в кеш
            save_in_cache(&cache_save_key, size, cache)?;

            size
        }
        app_arguments::CompressionArg::None => file
            .metadata()
            .wrap_err_with(|| format!("Failed to fetch metadata from {:?}", file_path))?
            .len(),
    };
    Ok(size)
}

//////////////////////////////////////////////////////////////////////////////////////////////////////

fn execute_app() -> Result<(), eyre::Error> {
    // Аргументы коммандной строки
    let arguments = app_arguments::AppArguments::from_args_safe().wrap_err("Arguments parsing")?;

    // Настройка логирования на основании количества флагов verbose
    setup_logging(&arguments).wrap_err("Logging setup")?;

    // Display arguments
    debug!("App arguments: {:?}", arguments);

    // Валидация параметров приложения
    validate_arguments(&arguments).wrap_err("Arguments validate")?;

    // Удаляем и создаем чистую конечную директорию
    remove_dir_all(&arguments.result_dirs_path).ok();
    create_dir_all(&arguments.result_dirs_path)
        .wrap_err_with(|| format!("Result directory create failed: {:?}", arguments.result_dirs_path))?;

    // Корзины со списками файлов
    let buckets = {
        let mut buckets = Vec::new();
        buckets.reserve(arguments.resuld_dirs_count);
        for i in 0..arguments.resuld_dirs_count {
            // Директории будут идти с единицы
            let result_directory = arguments.result_dirs_path.join((i + 1).to_string());
            let bucket = Bucked::new(result_directory).wrap_err("Bucket create")?;
            buckets.push(bucket);
        }
        buckets
    };

    // Система кешей
    let cache = match arguments.compression_type {
        CompressionArg::Brotli | CompressionArg::Gzip => {
            match arguments.compression_cache_path.as_ref() {
                Some(path) => {
                    // Создаем директории для кеша и открываем базу для хешей
                    create_dir_all(&path).wrap_err("Cache dir create failed")?;

                    let cache_db = sled::Config::default()
                        .path(path)
                        .mode(sled::Mode::HighThroughput)
                        .open()
                        .wrap_err("Cache db open failed")?;

                    Some(cache_db)
                }
                None => None,
            }
        }
        CompressionArg::None => None,
    };

    // Идем по всем входным директориям
    arguments
        .source_dirs
        .par_iter()
        .try_for_each(|source_dir| -> Result<(), eyre::Error> {
            // Получаем полный путь
            let full_source_dir = arguments.source_dirs_root.join(source_dir);

            // Обходим директорию параллельно
            WalkDir::new(full_source_dir)
                .into_iter()
                .par_bridge()
                .try_for_each(|entry| -> Result<(), eyre::Error> {
                    // Получаем путь
                    let file_path = entry.wrap_err("Access to entry failed")?.into_path();

                    // Работает лишь с файлами
                    if !file_path.is_file() {
                        return Ok(());
                    }

                    debug!("Found file path: {:?}", file_path);

                    // Размер данного файлика
                    let result_size = detect_result_file_size(&arguments.compression_type, arguments.compression_level, &file_path, &cache)
                        .wrap_err_with(|| format!("Compressed file size detection error: {:?}", file_path))?;

                    // Находим корзину с меньшим размером и кладем туда
                    let bucket = buckets.iter().min_by_key(|v| v.current_size()).wrap_err("Bucket find")?;

                    // Копируем файлик туда
                    bucket
                        .put_file(&arguments.source_dirs_root, &file_path, result_size)
                        .wrap_err_with(|| format!("Failed to put file: {:?}", file_path))?;

                    Ok(())
                })
        })?;

    Ok(())
}

fn main() {
    // Настройка color eyre для ошибок
    color_eyre::install().expect("Error setup failed");

    // Запуск приложения
    if let Err(err) = execute_app() {
        // При ошибке не паникуем, а спокойно выводим сообщение и завершаем приложение с кодом ошибки
        eprint!("Error! Failed with: {:?}", err);
        std::process::exit(1);
    }
}
