mod app_arguments;

use eyre::Context;
use image::{GenericImageView, ImageDecoder, ImageDecoderExt};
use lazy_static::lazy;
use lazy_static::lazy_static;
use log::{debug, error, info, trace};
use rayon::{iter::split, prelude::*};
use regex::Regex;
use serde::Serialize;
use std::{
    collections::hash_map::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read, Seek, Write},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
};
use structopt::StructOpt;
use walkdir::WalkDir;

struct CliAppPaths {
    identify_path: Option<PathBuf>,
    webpinfo_path: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct ImageSize {
    #[serde(rename = "w")]
    width: u32,

    #[serde(rename = "h")]
    height: u32,
}

fn get_file_type(file: &mut File) -> Result<Option<infer::Type>, eyre::Error> {
    let mut start_data_buffer = [0_u8; 16];
    if let Err(_) = file.read_exact(&mut start_data_buffer) {
        return Ok(None);
    }
    file.seek(std::io::SeekFrom::Start(0))
        .wrap_err("Cannot seek file")?;

    Ok(infer::get(&start_data_buffer))
}

lazy_static! {
    static ref RE: Regex = Regex::new(r#"Width:\s*(\d+)[\s\S]*Height:\s*(\d+)"#).unwrap();
}

fn try_get_image_size(
    path: &Path,
    cli_paths: &CliAppPaths,
) -> Result<Option<ImageSize>, eyre::Error> {
    if !path.is_file() {
        return Ok(None);
    }

    let mut file = File::open(&path).wrap_err_with(|| format!("Read file {:?}", path))?;

    let file_type = match get_file_type(&mut file).wrap_err("File type detection")? {
        Some(res) => res,
        None => return Ok(None),
    };

    //trace!("Image {:?} type: {:?}", path, infer_res);

    let file_ext = file_type.extension();

    match file_ext {
        "jpg" | "png" => {
            let reader = BufReader::new(file);

            let size = match file_ext {
                "jpg" => {
                    let decoder =
                        image::jpeg::JpegDecoder::new(reader).wrap_err("Decode jpeg file")?;
                    decoder.dimensions()
                }
                "png" => {
                    let decoder =
                        image::png::PngDecoder::new(reader).wrap_err("Decode png file")?;
                    decoder.dimensions()
                }
                _ => {
                    // TODO: Как-то лучше сделать, чтобы не надо было в рантайме
                    unreachable!("Must be valid type");
                }
            };

            Ok(Some(ImageSize {
                width: size.0,
                height: size.1,
            }))
        }
        "webp" => {
            drop(file);

            let file_path_str = path
                .to_str()
                .ok_or_else(|| eyre::eyre!("Cannot convert path to string"))?;

            if let Some(webpinfo_path) = &cli_paths.webpinfo_path {
                let output = Command::new(webpinfo_path)
                    .arg(file_path_str)
                    .stdin(Stdio::null())
                    .stderr(Stdio::piped())
                    .stdout(Stdio::piped())
                    .output()
                    .wrap_err("Webp info spawn")?;

                if output.status.success() {
                    let output_str =
                        std::str::from_utf8(&output.stdout).wrap_err("Webp stdout parse")?;

                    let cap = RE
                        .captures(output_str)
                        .ok_or_else(|| eyre::eyre!("Regex capture"))?;
                    let width = cap
                        .get(1)
                        .ok_or_else(|| eyre::eyre!("WebP width read failed: {}", output_str))?
                        .as_str()
                        .parse::<u32>()
                        .wrap_err_with(|| eyre::eyre!("WebP width parse failed: {}", output_str))?;

                    let height = cap
                        .get(2)
                        .ok_or_else(|| eyre::eyre!("WebP width read failed: {}", output_str))?
                        .as_str()
                        .parse::<u32>()
                        .wrap_err_with(|| eyre::eyre!("WebP width parse failed: {}", output_str))?;

                    Ok(Some(ImageSize { width, height }))
                } else {
                    let output_str =
                        std::str::from_utf8(&output.stderr).wrap_err("Webp stderr parse")?;
                    Err(eyre::eyre!(
                        "WebP wait failed with status {} and stderr: {}",
                        output.status,
                        output_str
                    ))
                }
            } else if let Some(identify_path) = &cli_paths.identify_path {
                let output = Command::new(identify_path)
                    .args(&["-ping", "-format", "%w,%h", file_path_str])
                    .stdin(Stdio::null())
                    .stderr(Stdio::piped())
                    .stdout(Stdio::piped())
                    .output()
                    .wrap_err("Webp identify spawn")?;

                if output.status.success() {
                    let output_str =
                        std::str::from_utf8(&output.stdout).wrap_err("Webp stdout parse")?;
                    let mut split_iter = output_str.split(",");
                    let width = split_iter
                        .next()
                        .ok_or_else(|| eyre::eyre!("WebP width read failed: {}", output_str))?
                        .parse::<u32>()
                        .wrap_err_with(|| eyre::eyre!("WebP width parse failed: {}", output_str))?;
                    let height = split_iter
                        .next()
                        .ok_or_else(|| eyre::eyre!("WebP out parse failed: {}", output_str))?
                        .parse::<u32>()
                        .wrap_err_with(|| eyre::eyre!("WebP width parse failed: {}", output_str))?;

                    Ok(Some(ImageSize { width, height }))
                } else {
                    let output_str =
                        std::str::from_utf8(&output.stderr).wrap_err("Webp stderr parse")?;
                    Err(eyre::eyre!(
                        "WebP wait failed with status {} and stderr: {}",
                        output.status,
                        output_str
                    ))
                }
            } else {
                Err(eyre::eyre!("Webp analyze application is missing"))
            }
        }
        _ => Ok(None),
    }
}

fn main() {
    human_panic::setup_panic!();
    color_eyre::install().expect("Error processing setup failed");

    let arguments = app_arguments::AppArguments::from_args();

    let filter_level = if arguments.verbose {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    };
    env_logger::builder().filter_level(filter_level).init();

    assert!(
        arguments.input_directory.exists(),
        "Input directory does not exist"
    );
    assert!(
        arguments.input_directory.is_dir(),
        "Input directory is not a folder"
    );

    let cli_app_paths = CliAppPaths {
        identify_path: which::which("identify").ok(),
        webpinfo_path: which::which("webpinfo").ok(),
    };

    let mut result: String = WalkDir::new(&arguments.input_directory)
        .into_iter()
        .par_bridge()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.into_path())
        .filter(|entry_path| {
            let ignored = arguments
                .ignore_directories
                .iter()
                .any(|ignore_dir| entry_path.starts_with(ignore_dir));
            !ignored
        })
        .filter_map(
            |entry_path| match try_get_image_size(&entry_path, &cli_app_paths) {
                Ok(Some(size)) => Some((entry_path, size)),
                Ok(None) => None,
                Err(err) => {
                    panic!("Image info error: {:?} for file: {:?}", err, entry_path);
                }
            },
        )
        .map(|(path, size)| {
            let val_str = serde_json::to_string(&size).expect("Serialize error");
            format!(
                "\"{key}\":{val},",
                key = path.to_str().unwrap(),
                val = val_str
            )
        })
        // Сборка идет по колонкам, поэтому начинаем просто с пустой строки, а не с '{'
        .reduce(
            || String::new(),
            |mut prev, next| {
                prev.push_str(&next);
                prev
            },
        );

    if result.ends_with(',') {
        // TODO: Может быть оптимальнее заменить последний символ?
        result.pop();
    }

    let mut out_file = File::create(arguments.output_file).expect("Output file open failed");
    out_file.write_all(&[b'{']).expect("Result write failed");
    out_file
        .write_all(result.as_bytes())
        .expect("Result write failed");
    out_file.write_all(&[b'}']).expect("Result write failed");

    // TODO: Validate result
}
