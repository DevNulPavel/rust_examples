// Таким вот образом подключаем внешние контейнеры (библиотеки)
extern crate lazy_static;
extern crate clap;
extern crate crossbeam;
extern crate crossbeam_channel;
extern crate num_cpus;

extern crate vulkan_shader_converter;

use vulkan_shader_converter::pre_process;
use vulkan_shader_converter::spirv_convert;

// Таким вот образом подключаем другие модули (файли dirs.rs или dirs/mod.rs)
// mod dirs;
// mod pre_process;
// mod spirv_convert;

fn print_invalid_params(){
    println!("Invalid parameters!");
}

fn main() {
    let matches = clap::App::new("Shader converter")
        .arg(clap::Arg::with_name("input")
                 .short("i")
                 .long("input")
                 .takes_value(true)
                 .help("input folder"))
        .arg(clap::Arg::with_name("output")
                 .short("o")
                 .long("output")
                 .takes_value(true)
                 .help("output folder"))
        .get_matches();

    // Проверяем наличие
    let out_res = matches.value_of("output");
    let input_res = matches.value_of("input");
    if out_res.is_none() || input_res.is_none(){
        print_invalid_params();
        return;
    }

    // Получаем значения
    let output_folder = out_res.unwrap();
    let input_folder = input_res.unwrap();

    // Получаем путь к временной папке
    let mut temp_folder_path: std::path::PathBuf  = std::env::temp_dir();
    temp_folder_path.push("vulkan_converter");
    // Удаляем, если существует уже
    if temp_folder_path.exists(){
        std::fs::remove_dir_all(&temp_folder_path).ok();
    }
    // Создаем временную директорию, если надо
    std::fs::create_dir(&temp_folder_path).unwrap();

    let input_folder_path = std::path::Path::new(input_folder);
    let out_folder_path = std::path::Path::new(output_folder);

    // Создаем конечную директорию, если надо
    if out_folder_path.exists(){
        std::fs::remove_dir_all(&out_folder_path).ok();
    }
    std::fs::create_dir(&out_folder_path).unwrap();

    // Выполняем предвариательную конвертацию
    pre_process::pre_process_shader_folder(&input_folder_path, &temp_folder_path).unwrap(); // &out_folder_path

    // Конвертация в SPIRV
    spirv_convert::convert_to_spirv_shaders(&temp_folder_path, &out_folder_path).unwrap();

    // Удаляем, если существует уже
    if temp_folder_path.exists(){
        std::fs::remove_dir_all(&temp_folder_path).unwrap();
    }
}