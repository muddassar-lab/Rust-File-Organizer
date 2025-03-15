use crate::models::CustomFile;
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn get_all_files(input_path: &PathBuf) -> Vec<CustomFile> {
    WalkDir::new(input_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| CustomFile::from_path(&entry.path().to_path_buf()))
        .collect()
}