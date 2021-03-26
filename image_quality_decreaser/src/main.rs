mod app_parameters;
mod image_file_type;
mod convert;

use std::{
    fs::{
        create_dir_all
    },
    path::{
        PathBuf,
        Path
    }
};
use walkdir::{
    WalkDir,
    DirEntry
};
use log::{
    //info,
    debug
};
use rayon::{
    prelude::{
        *
    }
};
use crate::{
    app_parameters::{
        AppParameters,
        parse_app_parameters
    },
    image_file_type::{
        ImageFileType,
        get_image_file_type
    },
    convert::{
        convert_webp,
        convert_with_image_magic
    }
};

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Если по указанному пути нету папки, тогда создаем ее
fn create_folder_if_does_not_exist(path: &Path){
    if !path.exists(){
        debug!("Try to create directory {:?}", path);
        create_dir_all(path)
            .expect("Directory create failed")
    }
}

fn make_result_file_path(input_file_path: &Path, input_folder: &Path, output_folder: &Path) -> PathBuf {
    let path_without_input_folder = input_file_path
        .strip_prefix(input_folder)
        .expect("Strip prefix failed");

    output_folder.join(path_without_input_folder)
}

fn process_file(app_params: &AppParameters, entry: DirEntry){
    // Получаем путь к конечному файлику
    let result_file_path = make_result_file_path(entry.path(), 
                                                    &app_params.input_folder, 
                                                    &app_params.output_folder);

    // Получаем папку конечного файла
    if let Some(result_file_folder) = result_file_path.parent(){
        create_folder_if_does_not_exist(result_file_folder);
    }

    // Получаем тип картинки или None если не картинка
    let file_type = get_image_file_type(entry.path());

    // Картинки конвертируем, обычные файлы просто копируем
    if let Some(file_type) = file_type{
        // Выполняем непосредственно конвертацию
        match file_type {
            ImageFileType::WebP => {
                convert_webp(entry.path(), &result_file_path, app_params.target_quality, app_params.resize_percent)
                    .expect("WebP convertation failed");
            },
            ImageFileType::Png | ImageFileType::Jpg => {
                convert_with_image_magic(entry.path(), &result_file_path, app_params.target_quality, app_params.resize_percent)
                   .expect("WebP/Png convertation failed");
            }
        }
    }else{
        std::fs::copy(entry.path(), result_file_path).expect("File copy failed");
    }
}

fn main(){
    env_logger::init();

    // Парсим параметры приложения
    let app_params = parse_app_parameters();

    // Создаем корневую папку для результатов если ее нету
    create_folder_if_does_not_exist(&app_params.output_folder);

    // Начинаем обходить входную директорию и обрабатывать входные пути
    WalkDir::new(&app_params.input_folder)
        .into_iter()
        .par_bridge()
        .filter_map(|entry|{
            entry.ok()
        })
        .filter(|entry|{
            entry.file_type().is_file()
        })
        .for_each(move |entry|{
            process_file(&app_params, entry)
            // debug!("Entry: {:#?}", entry)
        });
}