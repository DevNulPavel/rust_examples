use eyre::WrapErr;
use std::path::Path;
use tracing::instrument;

#[instrument(level = "error")]
pub fn create_dir_for_file(file_path: &Path) -> Result<(), eyre::Error> {
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).wrap_err_with(|| format!("target dir create failed: {:?}", parent))?;
    }
    Ok(())
}
