use std::{fs::{self, Metadata}, path::PathBuf};
use chrono::{DateTime, Local};
use super::FileType;

#[derive(Debug)]
pub struct CustomFile {
    pub extension: String,
    pub name: String,
    pub path: PathBuf,
    pub meta: Metadata,
}

impl CustomFile {
    pub fn from_path(path: &PathBuf) -> Option<Self> {
        let file_name = path.file_name()?.to_str()?;
        let extension = path.extension()?.to_str()?;

        let metadata = fs::metadata(path)
            .map_err(|e| {
                eprintln!("Failed to read metadata for '{}': {}", path.display(), e);
                e
            })
            .ok()?;

        Some(CustomFile {
            name: file_name.to_string(),
            extension: extension.to_string(),
            path: path.clone(),
            meta: metadata,
        })
    }

    pub fn get_type(&self) -> FileType {
        FileType::from_extension(&self.extension)
    }

    pub fn get_creation_date(&self) -> Result<String, String> {
        let created = self.meta.created().map_err(|e| {
            format!(
                "Failed to get creation time for '{}': {}",
                self.path.display(),
                e
            )
        })?;

        let datetime: DateTime<Local> = created.into();
        Ok(datetime.format("%Y-%m-%d").to_string())
    }
}