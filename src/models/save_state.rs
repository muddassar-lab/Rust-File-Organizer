use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{fs, io};

#[derive(Serialize, Deserialize)]
pub struct ProcessedFile {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub modified: std::time::SystemTime,
}

#[derive(Serialize, Deserialize)]
pub struct SaveState {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub processed_files: Vec<ProcessedFile>,
}

impl SaveState {
    pub fn new(input_path: PathBuf, output_path: PathBuf) -> Self {
        Self {
            input_path,
            output_path,
            processed_files: Vec::new(),
        }
    }

    pub fn add_processed_file(&mut self, path: PathBuf, name: String, size: u64, modified: std::time::SystemTime) {
        self.processed_files.push(ProcessedFile {
            path,
            name,
            size,
            modified,
        });
    }

    pub fn save_to_file(&self, file_path: &PathBuf) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(file_path, json)?;
        Ok(())
    }
}