use super::article::HabrArticle;
use std::{collections::HashSet, path::PathBuf};
use tokio::{fs::File}; // task::block_in_place

pub struct LoaderSaver {
    cache_file_name: PathBuf,
}

impl LoaderSaver {
    /// ".habrahabr_headers.json"
    pub fn new(file_name: &str) -> LoaderSaver {
        let temp_folder_path = dirs::home_dir().expect("Cannot get home folder directory path");

        let cache_file_path = std::path::PathBuf::new()
            .join(temp_folder_path)
            .join(file_name);

        LoaderSaver {
            cache_file_name: cache_file_path,
        }
    }

    pub async fn load_previous_results(&self) -> Option<HashSet<String>> {
        let file = File::open(&self.cache_file_name)
            .await
            .ok()?
            .into_std()
            .await;

        // let result =
            // block_in_place(move || serde_json::from_reader::<_, HashSet<String>>(file).ok());

        let result = serde_json::from_reader::<_, HashSet<String>>(file).ok();

        result
    }

    pub async fn save_links_to_file(&self, links: &[HabrArticle]) {
        let links_iter: Vec<&str> = links.iter().map(|info| info.link.as_str()).collect();

        let file = File::create(&self.cache_file_name)
            .await
            .expect("File save failed")
            .into_std()
            .await;

        // block_in_place(move || {
            serde_json::to_writer(file, &links_iter).ok();
        // });
    }
}
