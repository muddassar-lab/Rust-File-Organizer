use crate::utils::{ensure_save_dir, generate_save_filename, get_save_dir};
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

    pub fn add_processed_file(
        &mut self,
        path: PathBuf,
        name: String,
        size: u64,
        modified: std::time::SystemTime,
    ) {
        self.processed_files.push(ProcessedFile {
            path,
            name,
            size,
            modified,
        });
    }

    pub fn save(&self) -> io::Result<PathBuf> {
        ensure_save_dir()?;

        let save_dir = get_save_dir();
        let filename = generate_save_filename(&self.input_path);
        let save_path = save_dir.join(filename);

        let json = serde_json::to_string_pretty(self)?;
        fs::write(&save_path, json)?;

        Ok(save_path)
    }
}
