mod app_arguments;

use eyre::Context;
use flate2::read::GzDecoder;
use rayon::prelude::*;
use scopeguard::defer;
use std::{
    fs::File,
    io::{BufReader, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use structopt::StructOpt;
use walkdir::WalkDir;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

struct CliAppPaths {
    pvr_tex_tool_path: PathBuf,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ImageSize {
    width: u32,
    height: u32,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Декомпрессия pvrgz -> pvr
fn decompress_pvrgz_to_pvr(pvrgz_file_path: &Path, pvr_file_path: &Path) -> Result<(), eyre::Error> {
    let gz_file = File::open(pvrgz_file_path).wrap_err("Pvrgz open")?;
    let gz_reader = BufReader::new(gz_file);
    let mut gz_decoder = GzDecoder::new(gz_reader);

    let mut pvr_file = File::create(&pvr_file_path).wrap_err_with(|| format!("Temp PVR create failed: {:?}", pvr_file_path))?;

    std::io::copy(&mut gz_decoder, &mut pvr_file)
        .wrap_err_with(|| format!("Unzip failed from {:?} to {:?}", pvrgz_file_path, pvr_file_path))?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn pvrgz_image_size(pvrgz_file_path: &Path, temp_folder: &Path, cli_paths: &CliAppPaths) -> Result<ImageSize, eyre::Error> {
    // Имя временного PVR файлика в папке TMP
    let tmp_pvr_file_path = {
        let tmp = pvrgz_file_path.with_extension("pvr");
        let pvr_filename = tmp.file_name().unwrap();
        temp_folder.join(pvr_filename)
    };

    // Сразу планируем удаление
    defer!({
        std::fs::remove_file(tmp_pvr_file_path.clone()).ok();
    });

    // Распаковка PVRGZ в PVR во временной папке + чистка
    decompress_pvrgz_to_pvr(&pvrgz_file_path, &tmp_pvr_file_path)?;

    // Пути к временным png + out.pvr
    let tmp_pvr_file_decompressed_path = tmp_pvr_file_path.with_extension("out.pvr");
    let tmp_png_file_path = tmp_pvr_file_path.with_extension("png");

    // Сразу планируем удаление
    defer!({
        std::fs::remove_file(tmp_png_file_path.clone()).ok();
        std::fs::remove_file(tmp_pvr_file_decompressed_path.clone()).ok();
    });

    // Конвертация в png + out.pvr
    let output = Command::new(&cli_paths.pvr_tex_tool_path)
        .args(&[
            "-i",
            tmp_pvr_file_path.to_str().unwrap(),
            "-d",
            tmp_png_file_path.to_str().unwrap(),
            "-o",
            tmp_pvr_file_decompressed_path.to_str().unwrap(),
            "-f",
            "r4g4b4a4",
        ])
        .stderr(Stdio::piped())
        .stdout(Stdio::null())
        .stdin(Stdio::null())
        .output()
        .wrap_err("PVRTexTool spawn")?;

    // Проверка результатов вызова
    if !output.status.success() {
        let stderr_text = std::str::from_utf8(&output.stderr).wrap_err("PVRTexTool stderr parse")?;
        return Err(eyre::eyre!(
            "PVRTexTool failed with status '{}' and err: {}",
            output.status,
            stderr_text
        ));
    }

    // Чтобы точно закрыть дочерний процесс
    drop(output);

    // Размер из получившейся PNG
    let size = imagesize::size(&tmp_png_file_path)?;

    Ok(ImageSize {
        width: size.width as u32,
        height: size.height as u32,
    })
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// Попытка получения размера файлика
fn try_get_image_size(path: &Path, cli_paths: &CliAppPaths, temp_folder: &Path) -> Result<Option<ImageSize>, eyre::Error> {
    if !path.is_file() {
        return Ok(None);
    }

    let file_ext = match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => ext.to_lowercase(),
        None => return Ok(None),
    };

    match file_ext.as_str() {
        "jpg" | "png" | "webp" | "jpeg" => {
            let size = imagesize::size(path)?;
            Ok(Some(ImageSize {
                width: size.width as u32,
                height: size.height as u32,
            }))
        }
        "pvrgz" => {
            let res = pvrgz_image_size(path, temp_folder, cli_paths).wrap_err("Pvrgz convert")?;
            Ok(Some(res))
        }
        _ => Ok(None),
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn main() {
    human_panic::setup_panic!();
    color_eyre::install().expect("Error processing setup failed");

    let arguments = app_arguments::AppArguments::from_args();

    assert!(arguments.input_directory.exists(), "Input directory does not exist");
    assert!(arguments.input_directory.is_dir(), "Input directory is not a folder");

    let cli_app_paths = CliAppPaths {
        pvr_tex_tool_path: which::which("PVRTexToolCLI").expect("PVRTexToolCLI application is missing"),
    };

    let temp_folder = {
        let app_run_uuid = uuid::Uuid::new_v4().to_string();
        let temp_folder = std::env::temp_dir().join("images_info").join(app_run_uuid);
        if !temp_folder.exists() {
            std::fs::create_dir_all(&temp_folder).expect("Temp dir create failed");
        }
        temp_folder
    };

    defer!({
        std::fs::remove_dir(temp_folder.clone()).ok();
    });

    let result: String = WalkDir::new(&arguments.input_directory)
        .into_iter()
        .par_bridge()
        .filter_map(|entry| entry.ok())
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
        .filter_map(|entry_path| match try_get_image_size(&entry_path, &cli_app_paths, &temp_folder) {
            Ok(Some(size)) => Some((entry_path, size)),
            Ok(None) => None,
            Err(err) => {
                panic!("Image {:?} error: {:?} ", entry_path, err);
            }
        })
        // Конвертация в строку JSON
        .map(|(path, size)| {
            format!(
                "\"{key}\":{{\"w\":{w},\"h\":{h}}}",
                key = path.to_str().unwrap(),
                w = size.width,
                h = size.height
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

    // Пишем результат в файлик
    let mut out_file = File::create(arguments.output_file).expect("Output file open failed");
    out_file.write_all(&[b'{']).expect("Result write failed");
    out_file
        .write_all(result.trim_end_matches(',').as_bytes())
        .expect("Result write failed");
    out_file.write_all(&[b'}']).expect("Result write failed");

    // TODO: Validate result
}
