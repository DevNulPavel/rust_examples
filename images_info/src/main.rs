mod app_arguments;
mod pvr;
mod types;

use crate::{pvr::pvrgz_image_size, types::ImageSize};
use eyre::Context;
use rayon::prelude::*;
use std::{fs::File, io::Write, path::Path};
use structopt::StructOpt;
use walkdir::WalkDir;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// Попытка получения размера файлика
fn try_get_image_size(path: &Path) -> Result<Option<ImageSize>, eyre::Error> {
    // Файлик?
    if !path.is_file() {
        return Ok(None);
    }

    // Получаем расширение с нижнем регистре, либо выходим с None
    let file_ext = match path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_lowercase()) {
        Some(ext) => ext,
        None => return Ok(None),
    };

    match file_ext.as_str() {
        "jpg" | "png" | "webp" | "jpeg" => {
            // Получаем размер картинки для файлика
            let size = imagesize::size(path)?;
            Ok(Some(ImageSize {
                width: size.width as u32,
                height: size.height as u32,
            }))
        }
        "pvrgz" => {
            // Нашим собстенным способом читаем размер картинки из PVRGZ
            let res = pvrgz_image_size(path).wrap_err("Pvrgz convert")?;
            Ok(Some(res))
        }
        _ => Ok(None),
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn main() {
    // Человекочитаемый вывод паники
    human_panic::setup_panic!();

    // Поддержка Backtrace внутри ошибок
    color_eyre::install().expect("Error processing setup failed");

    // Аргументы коммандной строки
    let arguments = app_arguments::AppArguments::from_args();

    // Проверка входных параметров
    assert!(arguments.input_directory.exists(), "Input directory does not exist");
    assert!(arguments.input_directory.is_dir(), "Input directory is not a folder");

    // Запуск анализа директорий параллельно
    let result: String = WalkDir::new(&arguments.input_directory)
        .into_iter()
        // Параллельный итератор на пуле потоков
        .par_bridge()
        // Фильтруем только валидные директории
        .filter_map(|entry| entry.ok())
        // Конвертируем в путь
        .map(|entry| entry.into_path())
        // Фильтрация игнорируемых директорий
        .filter(|entry_path| {
            let ignored = arguments
                .ignore_directories
                .iter()
                .any(|ignore_dir| entry_path.starts_with(ignore_dir));
            !ignored
        })
        // Попытка получения размера + прерывание работы в случае ошибки
        .filter_map(|entry_path| match try_get_image_size(&entry_path) {
            Ok(Some(size)) => Some((entry_path, size)),
            Ok(None) => None,
            Err(err) => {
                panic!("\nImage {:?} error: {:?}\n", entry_path, err);
            }
        })
        // Конвертация в строку JSON
        .map(|(path, size)| {
            format!(
                "\"{key}\":{{\"w\":{w},\"h\":{h}}},",
                key = path.to_str().unwrap(),
                w = size.width,
                h = size.height
            )
        })
        // Вариант однопоточный
        /*.fold(String::new(), |mut prev, next| {
            prev.push_str(&next);
            prev
        });*/
        // Сборка идет группами в виде дерева, поэтому начинаем просто с пустой строки, а не с '{'
        .reduce(
            || String::new(),
            |mut prev, next| {
                prev.push_str(&next);
                prev
            },
        );

    // Пишем результат в файлик
    {
        let mut out_file = File::create(arguments.output_file).expect("Output file open failed");
        out_file.write_all(b"{").expect("Result write failed");
        out_file
            .write_all(result.trim_end_matches(',').as_bytes())
            .expect("Result write failed");
        out_file.write_all(b"}").expect("Result write failed");
    }

    // TODO: Validate result
}
