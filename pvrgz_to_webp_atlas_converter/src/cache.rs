use crate::helpers::create_dir_for_file;
use eyre::WrapErr;
use sled::Db;
use std::{
    fs::{copy, create_dir_all},
    path::{Path, PathBuf},
};
use tracing::{debug, instrument};

#[derive(Debug)]
pub struct CacheInfo {
    pub cache_db: Db,
    pub files_cache_dir: PathBuf,
}

impl CacheInfo {
    pub fn open(path: &Path) -> CacheInfo {
        // Создаем директории для кеша и открываем базу для хешей
        create_dir_all(&path).expect("Cache dir create failed");

        let files_cache_dir = path.join("files");
        create_dir_all(&files_cache_dir).expect("Cache dir create failed");

        let cache_db = sled::Config::default()
            .path(&path.join("hashes"))
            .mode(sled::Mode::HighThroughput)
            .open()
            .expect("Cache db open failed");

        CacheInfo { cache_db, files_cache_dir }
    }

    #[instrument(level = "error", skip(self))]
    pub fn save_file_in_cache(&self, key: &str, filepath: &Path) -> Result<(), eyre::Error> {
        // Копируем файлик в кеш и записываем в базу его uuid
        let uuid = uuid::Uuid::new_v4().to_string();
        let cached_file_path = self.files_cache_dir.join(&uuid);
        copy(filepath, cached_file_path).wrap_err("Copy file to cache")?;
        self.cache_db.insert(key, uuid.as_str()).wrap_err("Cache write failed")?;

        Ok(())
    }

    #[instrument(level = "error", skip(self))]
    pub fn try_restore_file_from_cache(&self, key: &str, target_file_path: &Path) -> Result<bool, eyre::Error> {
        if let Some(cached_file_name) = self.cache_db.get(&key).wrap_err("Db read error")? {
            // TODO: При ошибке просто конвертировать файлик, удалять старый и обновлять в базе данные

            // Путь к файлику кеша
            // TODO: Не парсить utf8?
            let cached_file_name_str = std::str::from_utf8(&cached_file_name).wrap_err("UTF-8 convert")?;
            let cached_file_path = self.files_cache_dir.join(cached_file_name_str);

            // Путь к файлику .webp
            create_dir_for_file(&target_file_path).wrap_err("Target file dir create error")?;

            // Копирование из кеша в нужную директорию
            std::fs::copy(cached_file_path, target_file_path).wrap_err("Cached file copy")?;

            debug!(?target_file_path, "Cache hit for file");

            Ok(true)
        } else {
            Ok(false)
        }
    }
}
