use crate::{cache::CacheInfo, helpers::get_md5_for_path, types::UtilsPathes};
use eyre::WrapErr;
use scopeguard::defer;
use std::{
    fs::{remove_file, File},
    io::copy,
    path::Path,
    process::{Command, Stdio},
};
use tracing::{instrument, trace, warn};

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

/// Возвращает путь к новому .webp файлику
#[instrument(level = "error", skip(cache_info, utils_pathes))]
pub fn pvrgz_to_webp(
    cache_info: &CacheInfo,
    utils_pathes: &UtilsPathes,
    target_webp_quality: u8,
    pvrgz_file_path: &Path,
) -> Result<(), eyre::Error> {
    // TODO: Использовать папку tmp?? Или не усложнять?

    // Путь к файлику .webp
    let webp_file_path = pvrgz_file_path.with_extension("webp");

    // Проверяем кеш файликов
    let pvrgz_md5 = get_md5_for_path(pvrgz_file_path).wrap_err("MD5 calculate")?;
    let full_pvrgz_path_str = pvrgz_file_path.to_str().ok_or_else(|| eyre::eyre!("Pvrgz full path str"))?;
    let cache_key = format!("{:x}_{}_{}", pvrgz_md5, full_pvrgz_path_str, target_webp_quality);
    if cache_info.try_restore_file_from_cache(&cache_key, &webp_file_path)? {
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

    // Конвертация .png -> .webp
    png_to_webp(&utils_pathes.cwebp, target_webp_quality, &png_file_path, &webp_file_path)
        .wrap_err_with(|| format!("{:?} -> {:?}", &png_file_path, &webp_file_path))?;

    // Копируем файлик в кеш и записываем в базу его uuid
    cache_info.save_file_in_cache(&cache_key, &webp_file_path).wrap_err("Cache save")?;

    Ok(())
}
