use eyre::WrapErr;
use std::path::{Path, PathBuf};

pub fn create_dir_for_file(file_path: &Path) -> Result<(), eyre::Error> {
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).wrap_err_with(|| format!("target dir create failed: {:?}", parent))?;
    }
    Ok(())
}

pub fn replace_root_on_path(absolute_file_path: &Path, source_root: &Path, target_root: &Path) -> Result<PathBuf, eyre::Error> {
    let relative_path = absolute_file_path.strip_prefix(source_root)?;
    let target_path = target_root.join(relative_path);

    Ok(target_path)
}
