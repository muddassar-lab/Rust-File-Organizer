use std::path::PathBuf;

#[derive(Debug)]
pub struct OrganizedFile {
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub file_name: String,
    pub size: u64,
}