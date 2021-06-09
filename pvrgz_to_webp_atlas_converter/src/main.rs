mod app_arguments;

use crate::app_arguments::AppArguments;
use eyre::WrapErr;
// use log::{debug, trace, warn};
use rayon::prelude::*;
use scopeguard::defer;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs::{remove_file, File},
    io::{copy, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    u8,
};
use structopt::StructOpt;
use tracing::{debug, instrument, trace, warn, Level};
use walkdir::WalkDir;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Настойка уровня логирования
/*fn setup_logging(arguments: &AppArguments) {
    // Настройка логирования на основании количества флагов verbose
    let level = match arguments.verbose {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        4 => log::LevelFilter::Trace,
        _ => {
            panic!("Verbose level must be in [0, 4] range");
        }
    };
    pretty_env_logger::formatted_builder()
        .filter_level(level)
        .try_init()
        .expect("Logger init failed");
}*/

/// Настойка уровня логирования
fn setup_logging(arguments: &AppArguments) {
    use tracing_subscriber::prelude::*;

    // Настройка логирования на основании количества флагов verbose
    let level = match arguments.verbose {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        3 => Level::TRACE,
        _ => {
            panic!("Verbose level must be in [0, 3] range");
        }
    };
    // tracing_subscriber::fmt()
    //     .with_target(false)
    //     .with_max_level(level)
    //     .try_init()
    //     .expect("Tracing init failed");
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::from_level(level))
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_error::ErrorLayer::default()) // Для поддержки захватывания SpanTrace в eyre
        .try_init()
        .expect("Tracing init failed");
}

/// Выполняем валидацию переданных аргументов приложения
fn validate_arguments(arguments: &AppArguments) {
    // Валидация параметров приложения
    assert!(
        arguments.atlases_images_directory.exists(),
        "Atlasses directory does not exist at path: {:?}",
        arguments.atlases_images_directory
    );
    assert!(arguments.target_webp_quality <= 100, "Target webp quality must be from 0 to 100");
    if let Some(alternative_atlases_json_directory) = arguments.alternative_atlases_json_directory.as_ref() {
        assert!(
            alternative_atlases_json_directory.exists(),
            "Atlasses alternative json directory does not exist at path: {:?}",
            alternative_atlases_json_directory
        );
    }
}

#[derive(Debug)]
pub struct UtilsPathes {
    pvr_tex_tool: PathBuf,
    cwebp: PathBuf,
}

#[derive(Debug)]
pub struct AtlasInfo {
    pvrgz_path: PathBuf,
    json_path: PathBuf,
}

#[instrument(level = "error")]
fn extract_pvrgz_to_pvr(pvrgz_file_path: &Path, pvr_file_path: &Path) -> Result<(), eyre::Error> {
    trace!(from = ?pvrgz_file_path, to = ?pvr_file_path, "Extract");

    // .pvrgz файлики
    let pvrgz_file = File::open(&pvrgz_file_path).wrap_err("Pvrgz open failed")?;
    let mut pvrgz_decoder = flate2::read::GzDecoder::new(pvrgz_file);

    // Путь к .pvr
    let mut pvr_file = File::create(&pvr_file_path).wrap_err("Pvr file create failed")?;

    // Извлекаем из .pvrgz в .pvr
    copy(&mut pvrgz_decoder, &mut pvr_file).wrap_err("Pvrgz extract failed")?;

    // Сразу же закроем файлики
    // drop(pvr_file);
    // drop(pvrgz_decoder);

    Ok(())
}

#[instrument(level = "error")]
fn pvr_to_png(pvr_tex_tool_path: &Path, pvr_file_path: &Path, png_file_path: &Path) -> Result<(), eyre::Error> {
    let pvr_tex_tool_output = Command::new(pvr_tex_tool_path)
        .args(&[
            "-ics",
            "sRGB",
            // "-f", "R4G4B4A4,USN",
            "-flip",
            "y",
            // "-p",
            "-i",
            pvr_file_path.to_str().ok_or_else(|| eyre::eyre!("Pvr path err"))?,
            "-d",
            png_file_path.to_str().ok_or_else(|| eyre::eyre!("Png path err"))?,
            "-noout",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .wrap_err("PVRTexToolCLI spawn failed")?;

    // Выводим ошибку и паникуем если не все хорошо
    if !pvr_tex_tool_output.status.success() {
        let err_output = std::str::from_utf8(&pvr_tex_tool_output.stderr).wrap_err("PVRTexToolCLI stderr parse failed")?;
        return Err(eyre::eyre!("PVRTexToolCLI stderr output: {}", err_output));
    }

    Ok(())
}

/*#[instrument(level = "error")]
fn png_premultiply_alpha(png_file_path: &Path) -> Result<(), eyre::Error> {
    let mut image = match image::open(png_file_path).wrap_err("Image open")? {
        image::DynamicImage::ImageRgba8(image) => image,
        _ => {
            warn!(path = ?png_file_path, "Is not RGBA8 image");
            return Ok(());
        }
    };

    debug!(?png_file_path, "Premultiply image alpha");
    image.pixels_mut().for_each(|pixel| {
        let alpha = (pixel[3] as f32) / 255.0_f32;
        pixel[0] = (pixel[0] as f32 * alpha) as u8;
        pixel[1] = (pixel[1] as f32 * alpha) as u8;
        pixel[2] = (pixel[2] as f32 * alpha) as u8;
    });

    image.save(png_file_path).wrap_err("Png save")?;

    Ok(())
}*/

#[instrument(level = "error")]
fn png_to_webp(cwebp_path: &Path, target_webp_quality: u8, png_file_path: &Path, webp_file_path: &Path) -> Result<(), eyre::Error> {
    let webp_tool_output = Command::new(&cwebp_path)
        .args(&[
            "-q",
            target_webp_quality.to_string().as_str(), // TODO: Optimize allocations
            "-o",
            webp_file_path.to_str().ok_or_else(|| eyre::eyre!("Webp path err"))?,
            png_file_path.to_str().ok_or_else(|| eyre::eyre!("Png path err"))?,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .wrap_err("PVRTexToolCLI spawn failed")?;

    // Выводим ошибку и паникуем если не все хорошо
    if !webp_tool_output.status.success() {
        let err_output = std::str::from_utf8(&webp_tool_output.stderr).wrap_err("cwebp stderr parse failed")?;
        return Err(eyre::eyre!("cwebp stderr output: {}", err_output));
    }

    Ok(())
}

#[instrument(level = "error")]
fn get_md5_for_file(path: &Path) -> Result<md5::Digest, eyre::Error> {
    let mut md5 = md5::Context::new();
    let mut file = File::open(path).wrap_err("File open")?;
    let mut buffer = [0_u8; 4096];
    loop {
        let read_count = file.read(&mut buffer)?;
        if read_count == 0 {
            break;
        }
        md5.consume(&buffer[0..read_count]);
    }
    Ok(md5.compute())
}

pub fn create_dir_for_file(file_path: &Path) -> Result<(), eyre::Error> {
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).wrap_err_with(|| format!("target dir create failed: {:?}", parent))?;
    }
    Ok(())
}

#[derive(Debug)]
struct CacheInfo {
    cache_db: sled::Db,
    files_cache_dir: PathBuf,
}

/// Возвращает путь к новому .webp файлику
#[instrument(level = "error", skip(cache_info, utils_pathes))]
fn pvrgz_to_webp(
    cache_info: &CacheInfo,
    utils_pathes: &UtilsPathes,
    target_webp_quality: u8,
    pvrgz_file_path: &Path,
) -> Result<(), eyre::Error> {
    // TODO: Использовать папку tmp?? Или не усложнять?

    let pvrgz_md5 = get_md5_for_file(pvrgz_file_path).wrap_err("MD5 calculate")?;
    let full_path = pvrgz_file_path.to_str().ok_or_else(|| eyre::eyre!("Pvrgz full path str"))?;
    let cache_key = format!("file:{}_md5:{:x}_q:{}", full_path, pvrgz_md5, target_webp_quality);
    if let Some(cached_file_name) = cache_info.cache_db.get(&cache_key).wrap_err("Db read error")? {
        // TODO: При ошибке просто конвертировать файлик, удалять старый и обновлять в базе данные

        // Путь к файлику кеша
        let cached_file_name_str = std::str::from_utf8(&cached_file_name)?;
        let cached_file_path = cache_info.files_cache_dir.join(cached_file_name_str);

        // Путь к файлику .webp
        let target_webp_file_path = pvrgz_file_path.with_extension("webp");
        create_dir_for_file(&target_webp_file_path).wrap_err("Target file dir create error")?;

        // Копирование из кеша в нужную директорию
        std::fs::copy(cached_file_path, target_webp_file_path).wrap_err("Cached file copy")?;

        debug!(?pvrgz_file_path, "Cache hit for file");

        return Ok(());
    }

    // Путь к временному .pvr
    let pvr_file_path = pvrgz_file_path.with_extension("pvr");
    defer!({
        // Запланируем сразу удаление файлика .pvr заранее
        if let Err(err) = remove_file(&pvr_file_path) {
            warn!(%err, "Temp pvr file remove failed: {:?}", pvr_file_path);
        }
    });

    // Извлекаем из .pvrgz в .pvr
    extract_pvrgz_to_pvr(pvrgz_file_path, &pvr_file_path).wrap_err_with(|| format!("{:?} -> {:?}", &pvrgz_file_path, &pvr_file_path))?;

    // Путь к файлику .png
    let png_file_path = pvr_file_path.with_extension("png");
    defer!({
        // Запланируем сразу удаление файлика .png заранее
        if let Err(err) = remove_file(&png_file_path) {
            warn!(%err, "Temp png file delete failed: {:?}", png_file_path);
        }
    });

    // Запуск конвертации .pvr в .png
    pvr_to_png(&utils_pathes.pvr_tex_tool, &pvr_file_path, &png_file_path)
        .wrap_err_with(|| format!("{:?} -> {:?}", &pvr_file_path, &png_file_path))?;

    // Для .png выполняем домножение альфы
    //png_premultiply_alpha(&png_file_path).wrap_err("Alpha premultiply")?;

    // Путь к файлику .webp
    let webp_file_path = png_file_path.with_extension("webp");

    // Конвертация .png -> .webp
    png_to_webp(&utils_pathes.cwebp, target_webp_quality, &png_file_path, &webp_file_path)
        .wrap_err_with(|| format!("{:?} -> {:?}", &png_file_path, &webp_file_path))?;

    // Копируем файлик в кеш и записываем в базу его uuid
    let uuid = uuid::Uuid::new_v4().to_string();
    let cached_file_path = cache_info.files_cache_dir.join(&uuid);
    std::fs::copy(webp_file_path, cached_file_path).wrap_err("Copy file to cache")?;
    cache_info
        .cache_db
        .insert(&cache_key, uuid.as_str())
        .wrap_err("Cache write failed")?;

    Ok(())
}

#[instrument(level = "debug")]
fn pvrgz_ext_to_webp(name: &mut String) -> Result<(), eyre::Error> {
    let mut new_file_name = name
        .strip_suffix(".pvrgz")
        .ok_or_else(|| eyre::eyre!("Json texture name must ends with .pvrgz"))?
        .to_owned();

    new_file_name.push_str(".webp");

    *name = new_file_name;

    Ok(())
}

#[instrument(level = "error")]
fn correct_file_name_in_json(cache_info: &CacheInfo, json_file_path: &Path) -> Result<(), eyre::Error> {
    #[derive(Debug, Deserialize, Serialize)]
    struct AtlasTextureMeta {
        #[serde(rename = "fileName")]
        file_name: Option<String>,
        #[serde(rename = "relPathFileName")]
        rel_file_name: Option<String>,
        #[serde(flatten)]
        other: Value,
    }
    #[derive(Debug, Deserialize, Serialize)]
    struct AtlasMetadata {
        #[serde(rename = "textureFileName")]
        texture_file_name: String,
        #[serde(flatten)]
        other: Value,
    }
    #[derive(Debug, Deserialize, Serialize)]
    struct AtlasMeta {
        texture: Option<AtlasTextureMeta>,
        metadata: Option<AtlasMetadata>,
        frames: Value,
        #[serde(flatten)]
        other: Value,
    }
    #[derive(Debug, Deserialize)]
    struct EmptyAtlasMeta {}
    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum FullMeta {
        Full(AtlasMeta),
        Empty(EmptyAtlasMeta),
    }

    let json_md5 = get_md5_for_file(json_file_path).wrap_err("MD5 calculate")?;
    let full_path = json_file_path.to_str().ok_or_else(|| eyre::eyre!("Json full path str"))?;
    let cache_key = format!("file:{}_md5:{:x}", full_path, json_md5);
    if let Some(cached_file_name) = cache_info.cache_db.get(&cache_key).wrap_err("Db read error")? {
        // TODO: При ошибке просто конвертировать файлик, удалять старый и обновлять в базе данные

        // Путь к файлику кеша
        let cached_file_name_str = std::str::from_utf8(&cached_file_name)?;
        let cached_file_path = cache_info.files_cache_dir.join(cached_file_name_str);

        // Путь к файлику .webp
        create_dir_for_file(&json_file_path).wrap_err("Target file dir create error")?;

        // Копирование из кеша в нужную директорию
        std::fs::copy(cached_file_path, json_file_path).wrap_err("Cached file copy")?;

        debug!(?json_file_path, "Cache hit for file");

        return Ok(());
    }

    let json_file = File::open(json_file_path).wrap_err("Json file open")?;

    let mut meta: AtlasMeta = match serde_json::from_reader(json_file).wrap_err("Json deserealize")? {
        FullMeta::Full(meta) => meta,
        FullMeta::Empty(_) => {
            warn!(?json_file_path, "Empty metadata at");
            return Ok(());
        }
    };

    // Может быть либо одно, либо другое
    if let Some(texture_info) = meta.texture.as_mut() {
        if let Some(name) = texture_info.file_name.as_mut() {
            pvrgz_ext_to_webp(name)?;
        } else if let Some(name) = texture_info.rel_file_name.as_mut() {
            pvrgz_ext_to_webp(name)?;
        } else {
            return Err(eyre::eyre!("Absolute or relative texture name must be specified"));
        }
    } else if let Some(metadata) = meta.metadata.as_mut() {
        pvrgz_ext_to_webp(&mut metadata.texture_file_name)?;
    } else {
        return Err(eyre::eyre!("Teture info or texture meta must be specified"));
    }

    let new_json_file = File::create(json_file_path).wrap_err("Result json file open")?;
    serde_json::to_writer(new_json_file, &meta).wrap_err("New json write failed")?;

    // Копируем файлик в кеш и записываем в базу его uuid
    let uuid = uuid::Uuid::new_v4().to_string();
    let cached_file_path = cache_info.files_cache_dir.join(&uuid);
    std::fs::copy(json_file_path, cached_file_path).wrap_err("Copy file to cache")?;
    cache_info
        .cache_db
        .insert(&cache_key, uuid.as_str())
        .wrap_err("Cache write failed")?;

    Ok(())
}

#[instrument(level = "error", skip(cache_info, utils_pathes))]
fn convert_pvrgz_atlas_to_webp(
    cache_info: &CacheInfo,
    utils_pathes: &UtilsPathes,
    target_webp_quality: u8,
    info: AtlasInfo,
) -> Result<(), eyre::Error> {
    // Из .pvrgz в .webp
    pvrgz_to_webp(cache_info, utils_pathes, target_webp_quality, &info.pvrgz_path).wrap_err("Pvrgz to webp convert")?;

    // Удаляем старый .pvrgz
    remove_file(&info.pvrgz_path).wrap_err("Pvrgz delete failed")?;

    // Правим содержимое .json файлика, прописывая туда .новое имя файла
    correct_file_name_in_json(cache_info, &info.json_path).wrap_err("Json fix failed")?;

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
        pvr_tex_tool: which::which("PVRTexToolCLI").expect("PVRTexTool application not found"),
        cwebp: which::which("cwebp").expect("PVRTexTool application not found"),
    };
    debug!(?utils_pathes, "Utils pathes");

    std::fs::create_dir_all(&arguments.cache_path).expect("Cache dir create failed");
    let files_cache_dir = arguments.cache_path.join("files");
    let cache_db = sled::Config::default()
        .path(&arguments.cache_path.join("hashes"))
        .mode(sled::Mode::HighThroughput)
        .open().expect("Cache db open failed");
    std::fs::create_dir_all(&files_cache_dir).expect("Cache dir create failed");
    let cache_info = CacheInfo {
        cache_db,
        files_cache_dir
    };

    WalkDir::new(&arguments.atlases_images_directory)
        // Параллельное итерирование
        .into_iter()
        // Параллелизация по потокам
        .par_bridge()
        // Только валидные папки и файлики
        .filter_map(|entry| entry.ok())
        // Конвертация в Path
        .map(|entry| entry.into_path())
        // Фильтруем только атласы
        .filter_map(|path| {
            // Это файлик .pvrgz?
            match path.extension().and_then(|ext| ext.to_str()) {
                Some("pvrgz") => {}
                _ => return None,
            }

            // Размер файла слишком мелкий? Тогда не трогаем - это может быть заглушка, либо это бессмысленно
            let meta = std::fs::metadata(&path).expect("File metadata read failed");
            if meta.len() < arguments.minimum_pvrgz_size {
                return None;
            }

            // Рядом с ним есть такой же .json?
            let same_folder_atlas_json_file = path.with_extension("json");
            if same_folder_atlas_json_file.exists() {
                // Возвращаем
                return Some(AtlasInfo {
                    pvrgz_path: path,
                    json_path: same_folder_atlas_json_file,
                });
            }

            // Может быть есть .json в отдельной директории?
            if let Some(alternative_atlases_json_directory) = arguments.alternative_atlases_json_directory.as_ref() {
                let relative_json_atlas_path = same_folder_atlas_json_file
                    .strip_prefix(&arguments.atlases_images_directory)
                    .expect("Images json prefix strip failed");
                let external_folder_atlas_json_file = alternative_atlases_json_directory.join(relative_json_atlas_path);
                if external_folder_atlas_json_file.exists() {
                    // Возвращаем
                    return Some(AtlasInfo {
                        pvrgz_path: path,
                        json_path: external_folder_atlas_json_file,
                    });
                }
            }

            None
        })
        // Непосредственно конвертация
        .for_each(|info| {
            debug!(?info, "Found atlas entry");

            if let Err(err) = convert_pvrgz_atlas_to_webp(&cache_info, &utils_pathes, arguments.target_webp_quality, info) {
                // При ошибке не паникуем, а спокойно выводим сообщение и завершаем приложение с кодом ошибки
                eprint!("Error! Failed with: {:?}", err);
                std::process::exit(1);
            }
        });
}
