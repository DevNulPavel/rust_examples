use crate::{cache::CacheInfo, helpers::get_md5_for_path};
use eyre::WrapErr;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs::File, path::Path, io::BufReader};
use tracing::{instrument, warn};

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

#[instrument(level = "error", skip(cache_info))]
pub fn correct_file_name_in_json(cache_info: &CacheInfo, json_file_path: &Path) -> Result<(), eyre::Error> {
    // Проверяем кеш файликов
    let json_md5 = get_md5_for_path(json_file_path).wrap_err("MD5 calculate")?;
    let full_json_path_str = json_file_path.to_str().ok_or_else(|| eyre::eyre!("Pvrgz full path str"))?;
    let cache_key = format!("{:x}_{}", json_md5, full_json_path_str);
    if cache_info.try_restore_file_from_cache(&cache_key, &json_file_path)? {
        return Ok(());
    }

    let json_file = File::open(json_file_path).wrap_err("Json file open")?;

    let mut meta: AtlasMeta = match serde_json::from_reader(BufReader::new(json_file)).wrap_err("Json deserealize")? {
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

    // TODO: Не открывать повторно файлик
    let new_json_file = File::create(json_file_path).wrap_err("Result json file open")?;
    serde_json::to_writer(new_json_file, &meta).wrap_err("New json write failed")?;

    // Копируем файлик в кеш и записываем в базу его uuid
    cache_info.save_file_in_cache(&cache_key, &json_file_path).wrap_err("Cache save")?;

    Ok(())
}
