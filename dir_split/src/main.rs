mod app_arguments;

use crate::app_arguments::{AppArguments, CompressionArg};
use eyre::{ContextCompat, WrapErr};
use log::{debug, LevelFilter};
use std::{
    fs::{copy as fs_copy, create_dir_all, remove_dir_all, File},
    io::{copy as io_copy, BufReader},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use structopt::StructOpt;
use walkdir::WalkDir;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Настойка уровня логирования
fn setup_logging(arguments: &AppArguments) -> Result<(), eyre::Error> {
    // Настройка логирования на основании количества флагов verbose
    let level = match arguments.verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        4 => LevelFilter::Trace,
        _ => {
            panic!("Verbose level must be in [0, 4] range");
        }
    };
    let logger = Box::new(pretty_env_logger::formatted_builder().filter_level(level).build());
    log::set_boxed_logger(logger)?;

    Ok(())
}

/// Выполняем валидацию переданных аргументов приложения
fn validate_arguments(arguments: &AppArguments) -> Result<(), eyre::Error> {
    // Валидация параметров приложения
    eyre::ensure!(arguments.resuld_dirs_count > 0, "Result dirs count must be greater than 0");
    eyre::ensure!(arguments.source_dirs.len() > 0, "Source directories count must be greater than 0");
    for path in arguments.source_dirs.as_slice() {
        eyre::ensure!(path.exists(), "Source directory does not exist: {:?}", path);
        eyre::ensure!(path.is_dir(), "Source directory must be dir: {:?}", path);
    }
    match arguments.use_compression {
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

        // Копируем
        fs_copy(&src_file_path, &result_full_path)
            .wrap_err_with(|| format!("File copy failed: {:?} -> {:?}", src_file_path, result_full_path))?;

        // Добавляем к конечному размеру размер результата, размер может быть после компрессии
        self.current_size.fetch_add(src_file_size, Ordering::SeqCst);

        Ok(())
    }
    fn current_size(&self) -> u64 {
        self.current_size.load(Ordering::SeqCst)
    }
}

fn detect_result_file_size(compression_type: &CompressionArg, compression_level: u8, file_path: &Path) -> Result<u64, eyre::Error> {
    let size = match compression_type {
        app_arguments::CompressionArg::Brotli => {
            let file = File::open(&file_path).wrap_err_with(|| format!("Source file open failed: {:?}", file_path))?;
            let mut reader = BufReader::new(file);

            let mut params = brotli::enc::BrotliEncoderParams::default();
            params.quality = compression_level as i32;

            let mut void_writer = std::io::sink();
            let size = brotli::BrotliCompress(&mut reader, &mut void_writer, &params).wrap_err("Brotli size analyze failed")?;

            size as u64
        }
        app_arguments::CompressionArg::Gzip => {
            let file = File::open(file_path).wrap_err_with(|| format!("Source file open failed: {:?}", file_path))?;
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
            let written = io_copy(&mut reader, &mut gz_writer).wrap_err_with(|| format!("Gzip compression failed: {:?}", file_path))?;
            written as u64
        }
        app_arguments::CompressionArg::None => {
            let file_size = file_path
                .metadata()
                .wrap_err_with(|| format!("Failed to fetch metadata from {:?}", file_path))?
                .len();
            file_size
        }
    };
    Ok(size)
}

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
            let result_directory = arguments.result_dirs_path.join(i.to_string());
            let bucket = Bucked::new(result_directory).wrap_err("Bucket create")?;
            buckets.push(bucket);
        }
        buckets
    };

    // Идем по всем выходным директориям и формируем постепенно корзины с равномерным заполнением
    // TODO: Использовать Rayon
    for source_dir in arguments.source_dirs {
        for entry in WalkDir::new(source_dir).into_iter() {
            // Получаем путь
            let file_path = entry.wrap_err("Access to entry failed")?.into_path();

            // Работает лишь с файлами
            if !file_path.is_file() {
                continue;
            }

            debug!("Found file path: {:?}", file_path);

            // Размер данного файлика
            // TODO: Кешировать размеры в базу данных на основании пути + MD5 + уровня компрессии
            // TODO: Определять сжатые размеры в пуле потоков
            let result_size = detect_result_file_size(&arguments.use_compression, arguments.compression_level, &file_path)
                .wrap_err_with(|| format!("Compressed file size detection error: {:?}", file_path))?;

            // Находим корзину с меньшим размером и кладем туда
            let bucket = buckets.iter().min_by_key(|v| v.current_size()).wrap_err("Bucket find")?;

            // Копируем файлик
            bucket
                .put_file(&arguments.source_dirs_root, &file_path, result_size)
                .wrap_err_with(|| format!("Failed to put file: {:?}", file_path))?;
        }
    }

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
