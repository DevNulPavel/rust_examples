use eyre::WrapErr;
use std::{fs::File, io::{Read, Seek, SeekFrom}, path::Path};
use tracing::instrument;

#[instrument(level = "error")]
pub fn create_dir_for_file(file_path: &Path) -> Result<(), eyre::Error> {
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).wrap_err_with(|| format!("target dir create failed: {:?}", parent))?;
    }
    Ok(())
}

#[instrument(level = "error")]
pub fn get_md5_for_path(path: &Path) -> Result<md5::Digest, eyre::Error> {
    let mut md5 = md5::Context::new();
    let mut file = File::open(path).wrap_err("File open")?;
    let mut buffer = [0_u8; 1024 * 16];
    loop {
        let read_count = file.read(&mut buffer)?;
        if read_count == 0 {
            break;
        }
        md5.consume(&buffer[0..read_count]);
    }
    Ok(md5.compute())
}

#[instrument(level = "error")]
pub fn get_md5_for_file(mut file: &File) -> Result<md5::Digest, eyre::Error> {
    let prev_pos = file.stream_position()?;

    file.seek(SeekFrom::Start(0))?;

    let mut md5 = md5::Context::new();
    let mut buffer = [0_u8; 1024 * 16];
    loop {
        let read_count = file.read(&mut buffer)?;
        if read_count == 0 {
            break;
        }
        md5.consume(&buffer[0..read_count]);
    }

    file.seek(SeekFrom::Start(prev_pos))?;

    Ok(md5.compute())
}