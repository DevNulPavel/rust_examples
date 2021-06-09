use std::path::PathBuf;

#[derive(Debug)]
pub struct UtilsPathes {
    pub pvr_tex_tool: PathBuf,
    pub cwebp: PathBuf,
}

#[derive(Debug)]
pub struct AtlasInfo {
    pub pvrgz_path: PathBuf,
    pub json_path: PathBuf,
}

#[derive(Debug)]
pub enum ConvertEntry {
    Atlas(AtlasInfo),
    OtherFile(PathBuf),
}
