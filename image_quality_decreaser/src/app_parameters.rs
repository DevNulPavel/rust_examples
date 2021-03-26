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
    pub target_quality: Option<u8>,
    pub resize_percent: Option<u16>
}

pub fn parse_app_parameters() -> AppParameters {
    let matches = App::new("Application for image quality decreasing")
        .version("1.0")
        .author("Pavel Ershov")
        .arg(Arg::with_name("input_folder")
                .short("i")
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
                .takes_value(true))
        .arg(Arg::with_name("resize_percent")
                .short("p")
                .long("resize_percent")
                .value_name("RESIZE_PERCENT")
                .help("Resize image in persents, example: --resize_percent 100")
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
    let target_quality = if let Some(entry) = matches.value_of("target_quality") {
        let val = entry
            .parse::<u8>()
            .expect("Target quality must be u8 integer value");
        assert!((val <= 100) && (val >= 1), "Quality value must be from 1 to 100");
        Some(val)
    }else{
        None
    };
    let resize_percent = if let Some(entry) = matches.value_of("resize_percent") {
        let val = entry
            .parse::<u16>()
            .expect("Resize percent must be u16 integer value");
        assert!(val >= 1, "Resize percent must be greater 1");
        Some(val)
    }else{
        None
    };   

    // Проверим переданные параметры с помощью ассертов
    assert!(input_folder.exists(), "Input folder does not exist!");
    assert!(input_folder.is_dir(), "Input folder must be folder!");
    assert!(output_folder.exists() == false, "Output folder already exists, remove it first!");

    AppParameters{
        input_folder,
        output_folder,
        target_quality,
        resize_percent
    }
}