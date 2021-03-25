use std::{
    path::{
        PathBuf
    }
};
use clap::{
    Arg, 
    App
};

pub struct AppParameters{
    pub input_folder: PathBuf,
    pub output_folder: PathBuf,
    pub target_quality: u8
}

pub fn parse_app_parameters() -> AppParameters {
    let matches = App::new("Application for image quality decreasing")
        .version("1.0")
        .author("Pavel Ershov")
        .arg(Arg::with_name("input_folder")
                .short("s")
                .long("input_folder")
                .value_name("INPUT_FOLDER")
                .help("Input folder for recursive image convertation")
                .required(true)
                .takes_value(true))
        .arg(Arg::with_name("output_folder")
                .short("o")
                .long("output_folder")
                .value_name("OUTPUT_FOLDER")
                .help("Result folder for converted images")
                .required(true)
                .takes_value(true))
        .arg(Arg::with_name("target_quality")
                .short("q")
                .long("target_quality")
                .value_name("TARGET_QUALITY")
                .help("Quality value of result images, value from 1 to 100")
                .required(true)
                .takes_value(true))               
        .get_matches();

    // Получаем входные значения в нужном формате
    let input_folder = matches
        .value_of("input_folder")
        .map(|path_str|{
            PathBuf::from(path_str)
        })
        .unwrap();
    let output_folder = matches
        .value_of("output_folder")
        .map(|path_str|{
            PathBuf::from(path_str)
        })
        .unwrap();
    let target_quality = matches
        .value_of("target_quality")
        .unwrap()
        .parse::<u8>()
        .expect("Target quality must be u8 integer value");

    // Проверим переданные параметры с помощью ассертов
    assert!((target_quality <= 100) && (target_quality >= 1), "Quality value must be from 1 to 100");
    assert!(input_folder.exists(), "Input folder does not exist!");
    assert!(input_folder.is_dir(), "Input folder must be folder!");
    assert!(output_folder.exists() == false, "Output folder already exists, remove it first!");

    AppParameters{
        input_folder,
        output_folder,
        target_quality
    }
}