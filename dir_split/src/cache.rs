use crate::{app_arguments::CompressionArg, helpers::get_md5_for_file};
use eyre::WrapErr;
use std::{convert::TryInto, fs::File, path::Path};

pub enum CacheResult {
    Found { size: u64 },
    NotFound { cache_save_key: String },
    NoCache,
}

pub fn search_in_cache(
    file_path: &Path,
    file: &mut File,
    compression_type: &CompressionArg,
    compression_level: u8,
    cache: &Option<sled::Db>,
) -> Result<CacheResult, eyre::Error> {
    if let Some(cache) = cache {
        let file_md5 = get_md5_for_file(file).wrap_err_with(|| format!("MD5 calc error: {:?}", file_path))?;
        let file_path_str = file_path
            .to_str()
            .ok_or_else(|| eyre::eyre!("Convert to str failed: {:?}", file_path))?;

        let key = format!("{:x}_{}_{}_{}", file_md5, file_path_str, compression_type, compression_level);

        let cached_val = cache.get(&key).wrap_err_with(|| format!("Size fetch failed: {}", key))?;
        match cached_val {
            Some(val) => {
                let bytes_ref: [u8; 8] = val.as_ref().try_into().wrap_err_with(|| format!("Invalid cached bytes: {}", key))?;
                let size = u64::from_be_bytes(bytes_ref);
                Ok(CacheResult::Found { size })
            }
            None => Ok(CacheResult::NotFound { cache_save_key: key }),
        }
    } else {
        Ok(CacheResult::NoCache)
    }
}

pub fn save_in_cache(key: &Option<String>, size: u64, cache: &Option<sled::Db>) -> Result<(), eyre::Error> {
    if let (Some(key), Some(cache)) = (key, cache) {
        let save_bytes = size.to_be_bytes();
        cache.insert(key, save_bytes.to_vec()).wrap_err("Cache size save failed")?;
    }
    Ok(())
}
