mod app_arguments;
mod types;

use crate::{
    app_arguments::AppArguments,
    types::{PackConfig, PackData, PackFilePathInfo},
};
use fancy_regex::Regex;
use log::{debug, error, info, trace, warn};
use rayon::prelude::*;
use std::{
    collections::HashMap,
    fmt::format,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
use structopt::StructOpt;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Настойка уровня логирования
fn setup_logging(arguments: &AppArguments) {
    // Настройка логирования на основании количества флагов verbose
    let level = match arguments.verbose {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        3 => log::LevelFilter::Trace,
        _ => {
            panic!("Verbose level must be in [0, 3] range");
        }
    };
    pretty_env_logger::formatted_builder()
        .filter_level(level)
        .try_init()
        .expect("Logger init failed");
}

/// Выполняем валидацию переданных аргументов приложения
fn validate_arguments(arguments: &AppArguments) {
    // Валидация параметров приложения
    assert!(
        arguments.dynamic_packs_config_path.exists(),
        "Dynamic packs config does not exist at path: {:?}",
        arguments.dynamic_packs_config_path
    );
    assert!(arguments.dynamic_packs_config_path.is_file(), "Dynamic packs config is not a file");
    assert!(
        arguments.resources_directory.exists(),
        "Resources directory does not exist: {:?}",
        arguments.resources_directory
    );
    assert!(arguments.resources_directory.is_dir(), "Resources directory is not a directory");
}

struct PackIter<'a, I: Iterator<Item = PathBuf>> {
    source: I,
    sub_pack_number: u32,
    max_pack_size: u64,
    pack_config: &'a PackConfig,
    resources_root_folder: &'a Path,
}
impl<'a, I: Iterator<Item = PathBuf>> Iterator for PackIter<'a, I> {
    type Item = PackData;
    fn next(&mut self) -> Option<Self::Item> {
        // Список файликов пака
        let mut pack_files = Vec::new();
        // Размер пака
        let mut cur_pack_size = 0;

        'internal_loop: loop {
            // Получаем из итератора путь к файлику, но не изымаем из итератора
            let file_full_path = match self.source.next() {
                Some(path) => {
                    // trace!("Peek path: {:?}", path);
                    path
                }
                None => {
                    // trace!("All files iter break");
                    // break 'top_loop;
                    return None;
                }
            };

            // Читаем метаданные файлика
            let meta = std::fs::metadata(&file_full_path).expect("Metadata read failed");

            // Увеличиваем размер
            cur_pack_size = cur_pack_size + meta.len();

            // Закешируем относительный путь
            let relative_path = file_full_path
                .strip_prefix(self.resources_root_folder)
                .expect("Resources root strip failed")
                .to_str()
                .expect("Convert to string failed")
                .to_owned();

            // Записываем
            pack_files.push(PackFilePathInfo {
                absolute: file_full_path,
                relative: relative_path,
            });

            // Если размер уже переполнен, тогда прерываем внутренний цикл
            if cur_pack_size >= self.max_pack_size {
                break 'internal_loop;
            }
        }

        // Пустой пак?
        // if pack_files.len() == 0 {
        //     continue 'top_loop;
        // }

        // Записываем информацию о паке
        let res = PackData {
            files_paths: pack_files,
            pack_name: format!("{}_{}", self.pack_config.name, self.sub_pack_number),
            required: self.pack_config.required,
            priority: self.pack_config.priority,
            pack_size: cur_pack_size,
        };

        // Номер под-пака увеличиваем на 1
        self.sub_pack_number = self.sub_pack_number + 1;

        Some(res)
    }
}

fn pack_info_for_config<'a>(
    max_pack_size: u64,
    resources_root_folder: &'a Path,
    pack_config: &'a PackConfig,
) -> impl 'a + Iterator<Item = PackData> {
    // Компилируем переданные регулярные выражения
    let resource_regex_filters: Vec<Regex> = pack_config
        .resources
        .iter()
        .map(|resource_regex| Regex::new(&resource_regex).expect("Regex create failed"))
        .collect();

    // На основании регулярок получаем список подходящих директорий
    let all_files_iter = walkdir::WalkDir::new(resources_root_folder)
        .into_iter()
        .map(|entry| entry.expect("WalkDir entry unwrap failed"))
        .filter(|entry| entry.path().is_dir())
        .filter(move |entry| {
            // trace!("Folder check: {:?}", entry.path());
            let entry_path_str = entry
                .path()
                .strip_prefix(resources_root_folder)
                .expect("Strip prefix error")
                .to_str()
                .expect("Entry to string convert error");
            resource_regex_filters
                .iter()
                .any(|regex| regex.is_match(entry_path_str).expect("Regex check failed"))
        })
        // Итератор списка всех файликов в директориях конфига пака
        .map(|entry| walkdir::WalkDir::new(entry.path()))
        .flatten()
        .map(|entry| entry.expect("WalkDir entry unwrap failed").into_path())
        .filter(|path| path.is_file());

    PackIter {
        resources_root_folder,
        max_pack_size,
        pack_config,
        source: all_files_iter,
        sub_pack_number: 0,
    }
}

fn create_pack_zip(result_packs_folder: &Path, pack: &PackData) {
    let result_pack_path = result_packs_folder.join(format!("{}.dpk", pack.pack_name));
    trace!(
        "Zip file write (thread {:?}): {:?}",
        rayon::current_thread_index(),
        result_pack_path
    );

    let file = File::create(&result_pack_path).expect("Result file open failed");
    let mut zip = zip::ZipWriter::new(file);

    // Defer delete if error
    scopeguard::defer_on_unwind!({
        std::fs::remove_file(result_pack_path).ok();
    });

    pack.files_paths.iter().for_each(|path_info| {
        let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated); // TODO: Params
        zip.start_file(&path_info.relative, options).expect("Create zip file err");

        let mut original_file = File::open(&path_info.absolute).expect("Original file open err");
        std::io::copy(&mut original_file, &mut zip).expect("File copy failed"); // TODO: Bufffer size
    });

    zip.finish().expect("Zip file write failed");
}

fn main() {
    // Человекочитаемый вывод паники
    color_backtrace::install();

    // Аргументы коммандной строки
    let arguments = app_arguments::AppArguments::from_args();

    // Настройка логирования на основании количества флагов verbose
    setup_logging(&arguments);

    // Display arguments
    debug!("App arguments: {:#?}", arguments);

    // Валидация параметров приложения
    validate_arguments(&arguments);

    // Пасим json конфиг
    let config_file = File::open(&arguments.dynamic_packs_config_path).expect("Dynamic packs config file open failed");
    let packs_configs: Vec<PackConfig> = serde_json::from_reader(config_file).expect("Dynamic packs config parse failed");
    debug!("Packs config: {:#?}", packs_configs);

    // Создаем директорию для паков если надо
    std::fs::create_dir_all(&arguments.output_dynamic_packs_dir).expect("Output packs directory create failed");

    // Делим переданные нам конфиги на паки
    let result_packs_infos: Vec<PackData> = packs_configs
        .par_iter()
        .flat_map_iter(|pack_config| pack_info_for_config(arguments.max_pack_size, &arguments.resources_directory, pack_config) )
        .collect();
    drop(packs_configs);
    debug!("Result packs infos: {:?}", result_packs_infos);

    // Создаем .dpk архивы
    result_packs_infos.par_iter().for_each(|pack_info| {
        // debug!("Pack info: {:#?}", pack_info);
        create_pack_zip(&arguments.output_dynamic_packs_dir, &pack_info)
    });

    // Создаем директорию для конечного конфига
    if let Some(parent) = arguments.output_resources_config_path.parent() {
        std::fs::create_dir_all(parent).expect("Output resources config create failed");
    }

    // Создаем конечный конфиг
    let mut result_config_file = File::create(&arguments.output_resources_config_path).expect("Result config file open failed");
    let resources_str: String = result_packs_infos
        .par_iter()
        .fold_with(String::new(), |mut prev, config| {
            config.files_paths.iter().for_each(|path_info| {
                let line = format!(r#""{}":"{}","#, path_info.relative, config.pack_name);
                prev.push_str(&line);
            });
            prev
        })
        .collect();
    result_config_file.write_all(br#"{"resources":{"#).unwrap();
    result_config_file
        .write_all(resources_str.trim_end_matches(",").as_bytes())
        .unwrap();
    drop(resources_str);
    result_config_file.write_all(br#"},"packs":["#).unwrap();
    let packs_str: String = result_packs_infos
        .par_iter()
        .fold_with(String::new(), |mut prev, config| {
            let dpk_path = arguments.config_pack_file_dir.join(format!("{}.dpk", config.pack_name));
            let line = format!(
                r#"{{"name":"{}","path":"{}","priority":{},"required":{}}},"#,
                config.pack_name, dpk_path.to_str().unwrap(), config.priority, config.required
            );
            prev.push_str(&line);
            prev
        })
        .collect();
    result_config_file.write_all(packs_str.trim_end_matches(",").as_bytes()).unwrap();
    drop(packs_str);
    result_config_file.write_all(br#"]}"#).unwrap();
}
