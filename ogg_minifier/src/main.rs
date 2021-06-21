mod app_arguments;
mod helpers;
mod types;

use crate::{
    app_arguments::{AppArguments, TargetParams},
    types::UtilsPathes,
};
use eyre::WrapErr;
use rayon::prelude::*;
use std::{
    cmp::min,
    fs::{remove_file, rename, File},
    path::PathBuf,
    process::{Command, Stdio},
};
use structopt::StructOpt;
use tracing::{debug, instrument, Level};
use walkdir::WalkDir;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Настойка уровня логирования
fn setup_logging(arguments: &AppArguments) {
    use tracing_subscriber::prelude::*;

    // Настройка логирования на основании количества флагов verbose
    let level = match arguments.verbose {
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        4 => Level::TRACE,
        _ => {
            panic!("Verbose level must be in [0, 4] range");
        }
    };
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::from_level(level))
        .with(tracing_subscriber::filter::EnvFilter::new(env!("CARGO_PKG_NAME"))) // Логи только от текущего приложения, без библиотек
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_error::ErrorLayer::default()) // Для поддержки захватывания SpanTrace в eyre
        .try_init()
        .expect("Tracing init failed");
}

/// Выполняем валидацию переданных аргументов приложения
fn validate_arguments(arguments: &AppArguments) {
    // Валидация параметров приложения
    assert!(
        arguments.ogg_files_directory.exists(),
        "Input files directory does not exist at path: {:?}",
        arguments.ogg_files_directory
    );
}

#[instrument(level = "error", skip(utils_pathes))]
fn convert_ogg_file(utils_pathes: &UtilsPathes, params: &TargetParams, ogg_file_path: PathBuf) -> Result<(), eyre::Error> {
    // Читаем метаинформацию о файлике .ogg
    let ogg_file = File::open(&ogg_file_path).wrap_err("Ogg file first open")?;
    let mut ogg_reader = ogg::PacketReader::new(ogg_file);
    let ((audio_info, _, _), _) = lewton::inside_ogg::read_headers(&mut ogg_reader).wrap_err("Ogg headers read failed")?;
    drop(ogg_reader);
    debug!(
        freq = audio_info.audio_sample_rate,
        bitrate_maximum = audio_info.bitrate_maximum,
        bitrate_nominal = audio_info.bitrate_nominal,
        bitrate_minimum = audio_info.bitrate_minimum,
        "Ogg headers"
    );

    // Может криво определились значения?
    eyre::ensure!(audio_info.bitrate_nominal > 1, "Nominal ogg bitrate is not positive value");

    // Если все параметры удовлетворяют максимуму, значит оставляем как есть
    if audio_info.bitrate_nominal <= params.max_bitrate as i32 && audio_info.audio_sample_rate <= params.max_freq {
        debug!("Everything is ok, do not convert file");
        return Ok(());
    }

    // Если нет, находим целевые значения
    let target_bitrate = min(audio_info.bitrate_nominal, params.max_bitrate as i32);
    let target_freq = min(audio_info.audio_sample_rate, params.max_freq);

    // Делаем их валидацию
    eyre::ensure!(target_bitrate > 1, "New bitrate must be valid positive value");
    eyre::ensure!(target_freq > 1, "New freq must be valid positive value");

    // Путь к новому временному файлику
    let file_name_str = ogg_file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| eyre::eyre!("Filename receive failed"))?;
    let tmp_file_path = ogg_file_path
        .parent()
        .ok_or_else(|| eyre::eyre!("File path parent receive error"))?
        .join(format!("tmp_{}", file_name_str));
    let tmp_file_path_str = tmp_file_path.to_str().ok_or_else(|| eyre::eyre!("Temp file path to str error"))?;

    // Путь к исходному файлу в виде строки
    let ogg_file_path_str = ogg_file_path.to_str().ok_or_else(|| eyre::eyre!("File path to str error"))?;

    debug!(target_bitrate, target_freq, "Convert to new params");

    // Вызываем конвертацию ffmpeg
    // https://askubuntu.com/a/1119133
    #[rustfmt::skip]
    let pvr_tex_tool_output = Command::new(&utils_pathes.ffmpeg)
        .args(&[
            "-i", ogg_file_path_str,
            "-c:a", "libvorbis", 
            "-ab", &target_bitrate.to_string(),
            "-ar", &target_freq.to_string(),
            tmp_file_path_str
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .wrap_err("ffmpeg spawn failed")?;

    // Выводим ошибку
    if !pvr_tex_tool_output.status.success() {
        let err_output = std::str::from_utf8(&pvr_tex_tool_output.stderr).wrap_err("ffmpeg stderr parse failed")?;
        return Err(eyre::eyre!("ffmpeg stderr output: {}", err_output));
    }

    // Удаляем старый файлик
    remove_file(&ogg_file_path).wrap_err("Old file remove")?;

    // Переименование в новый файлик
    rename(tmp_file_path, ogg_file_path).wrap_err("Temp file rename")?;

    Ok(())
}

fn main() {
    // Человекочитаемый вывод паники
    color_backtrace::install();

    // Настройка color eyre для ошибок
    color_eyre::install().expect("Error setup failed");

    // Аргументы коммандной строки
    let arguments = app_arguments::AppArguments::from_args();

    // Настройка логирования на основании количества флагов verbose
    setup_logging(&arguments);

    // Display arguments
    debug!(?arguments, "App arguments");

    // Валидация параметров приложения
    validate_arguments(&arguments);

    // Находим пути к бинарникам для конвертации
    let utils_pathes = UtilsPathes {
        ffmpeg: which::which("ffmpeg").expect("ffmpeg application not found"),
    };
    debug!(?utils_pathes, "Utils pathes");

    WalkDir::new(&arguments.ogg_files_directory)
        // Параллельное итерирование
        .into_iter()
        // Параллелизация по потокам
        .par_bridge()
        // Только валидные папки и файлики
        .filter_map(|entry| entry.ok())
        // Конвертация в Path
        .map(|entry| entry.into_path())
        // Фильтруем только атласы
        .filter(|path| {
            // Это файлик .ogg?
            match path.extension().and_then(|ext| ext.to_str()) {
                Some("ogg") => true,
                _ => false,
            }
        })
        // Если кто-то запаниковал, тогда останавливаем работу остальных потоков
        .panic_fuse()
        // Непосредственно конвертация
        .for_each(|path| {
            debug!(?path, "Found entry");

            if let Err(err) = convert_ogg_file(&utils_pathes, &arguments.target_params, path) {
                // При ошибке не паникуем, а спокойно выводим сообщение и завершаем приложение с кодом ошибки
                eprint!("Error! Failed with: {:?}", err);
                std::process::exit(1);
            }
        });
}
