mod app_arguments;
mod types;

use crate::{
    app_arguments::AppArguments,
    types::{PackConfig, PackData},
};
use fancy_regex::Regex;
use log::{debug, error, info, trace, warn};
use rayon::prelude::*;
use std::{collections::HashMap, fs::File, io::Write, path::Path};
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

fn pack_info_for_config(max_pack_size: u64, resources_root_folder: &Path, pack_config: &PackConfig) -> Vec<PackData> {
    // Компилируем переданные регулярные выражения
    let resource_regex_filters: Vec<Regex> = pack_config
        .resources
        .iter()
        .map(|resource_regex| Regex::new(&resource_regex).expect("Regex create failed"))
        .collect();

    // На основании регулярок получаем список подходящих директорий
    let mut all_files_iter = walkdir::WalkDir::new(resources_root_folder)
        .into_iter()
        // .par_bridge()
        .map(|entry| entry.expect("WalkDir entry unwrap failed"))
        .filter(|entry| entry.path().is_dir())
        .filter(|entry| {
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
        .map(|entry|{
            entry.expect("WalkDir entry unwrap failed").into_path()
        })
        .filter(|path| path.is_file() );

    // Результат
    let mut result = Vec::new();
    let mut sub_pack_number = 0;
    'top_loop: loop {
        // Список файликов пака
        let mut pack_files = Vec::new();
        // Размер пака
        let mut cur_pack_size = 0;

        'internal_loop: loop {
            // Получаем из итератора путь к файлику, но не изымаем из итератора
            let file_path = match all_files_iter.next() {
                Some(path) => {
                    trace!("Peek path: {:?}", path);
                    path
                },
                None => {
                    trace!("All files iter break");
                    break 'top_loop;
                }
            };

            // Читаем метаданные файлика
            let meta = std::fs::metadata(&file_path).expect("Metadata read failed");

            // Увеличиваем размер
            cur_pack_size = cur_pack_size + meta.len();

            // Записываем
            pack_files.push(file_path);

            // Если размер уже переполнен, тогда прерываем внутренний цикл
            if cur_pack_size >= max_pack_size {
                break 'internal_loop;
            }
        }

        // Пустой пак?
        if pack_files.len() == 0 {
            continue 'top_loop;
        }

        // Записываем информацию о паке
        result.push(PackData {
            files: pack_files,
            pack_name: format!("{}_{}", pack_config.name, sub_pack_number),
            required: pack_config.required,
            priority: pack_config.priority,
            pack_size: cur_pack_size,
        });

        // Номер под-пака увеличиваем на 1
        sub_pack_number = sub_pack_number + 1;
    }

    result
}

/// На основании списка конфигов для паков формируем хэшмап с информацией о паках
fn generate_pack_infos(max_pack_size: u64, resources_root_folder: &Path, pack_configs: Vec<PackConfig>) -> Vec<PackData> {
    pack_configs
        .par_iter()
        .map(|pack_config| pack_info_for_config(max_pack_size, resources_root_folder, pack_config))
        .flatten()
        .collect()
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
    let config_file = File::open(arguments.dynamic_packs_config_path).expect("Dynamic packs config file open failed");
    let packs_configs: Vec<PackConfig> = serde_json::from_reader(config_file).expect("Dynamic packs config parse failed");
    debug!("Packs config: {:#?}", packs_configs);

    // Делим переданные нам конфиги на паки
    let pack_infos = generate_pack_infos(arguments.max_pack_size, &arguments.resources_directory, packs_configs);
    debug!("Packs infos: {:#?}", pack_infos);
}
