mod app_arguments;

use log::{debug, error, info, trace, warn};
use rayon::prelude::*;
use std::{fs::File, io::Write, path::Path};
use structopt::StructOpt;
use walkdir::WalkDir;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn main() {
    // Человекочитаемый вывод паники
    human_panic::setup_panic!();

    // Аргументы коммандной строки
    let arguments = app_arguments::AppArguments::from_args();

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

    info!("App arguments: {:#?}", arguments);

    error!("test");
    warn!("test");
    info!("test");
    debug!("test");
    trace!("test");
}
