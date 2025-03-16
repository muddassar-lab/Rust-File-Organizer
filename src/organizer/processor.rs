use crate::error::OrganizeError;
use crate::models::{CustomFile, OrganizedFile, SaveState};
use rayon::prelude::*;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const BUFFER_SIZE: usize = 8192;

pub fn organize_files(
    files: Vec<CustomFile>,
    output_path: &PathBuf,
) -> Result<Vec<OrganizedFile>, OrganizeError> {
    files
        .par_iter() // Using rayon for parallel processing
        .map(|file| {
            let file_type = file.get_type();
            let date = file
                .get_creation_date()
                .map_err(|e| OrganizeError::UserInputError(e))?;

            let type_dir = output_path.join(format!("{:?}", file_type));
            let date_dir = type_dir.join(date);

            fs::create_dir_all(&date_dir)
                .map_err(|e| OrganizeError::DirectoryCreationFailed(e.to_string()))?;

            let target_path = date_dir.join(&file.name);

            Ok(OrganizedFile {
                source_path: file.path.clone(),
                target_path,
                file_name: file.name.clone(),
                size: file.meta.len(),
            })
        })
        .collect()
}

pub fn copy_files<F>(
    organized_files: Vec<OrganizedFile>,
    mut progress_callback: F,
    stop_signal: Arc<AtomicBool>,
) -> Result<Option<SaveState>, OrganizeError>
where
    F: FnMut(&str, u64, u64, usize),
{
    let mut save_state = SaveState::new(
        organized_files
            .first()
            .map(|f| {
                f.source_path
                    .parent()
                    .unwrap_or(&f.source_path)
                    .to_path_buf()
            })
            .unwrap_or_default(),
        organized_files
            .first()
            .map(|f| {
                f.target_path
                    .parent()
                    .unwrap_or(&f.target_path)
                    .to_path_buf()
            })
            .unwrap_or_default(),
    );

    for (index, file) in organized_files.into_iter().enumerate() {
        if stop_signal.load(Ordering::SeqCst) && index > 0 {
            return Ok(Some(save_state));
        }

        // Copy file with progress
        copy_file_with_progress(
            &file.source_path,
            &file.target_path,
            &file.file_name,
            file.size,
            |bytes_copied| {
                progress_callback(&file.file_name, file.size, bytes_copied, index + 1);
            },
        )?;

        // Call progress callback with final state
        progress_callback(&file.file_name, file.size, file.size, index);
        let source_path = file.source_path.clone();

        // Add to save state
        save_state.add_processed_file(
            file.source_path,
            file.file_name,
            file.size,
            std::fs::metadata(source_path)
                .map_err(|e| OrganizeError::FileCopyFailed(e.to_string()))?
                .modified()
                .unwrap_or_else(|_| std::time::SystemTime::now()),
        );
    }

    Ok(None)
}

fn copy_file_with_progress<F>(
    source: &PathBuf,
    target: &PathBuf,
    _file_name: &str,
    file_size: u64,
    mut progress_callback: F,
) -> Result<(), OrganizeError>
where
    F: FnMut(u64),
{
    let mut source_file =
        File::open(source).map_err(|e| OrganizeError::FileCopyFailed(e.to_string()))?;
    let mut target_file =
        File::create(target).map_err(|e| OrganizeError::FileCopyFailed(e.to_string()))?;

    let mut buffer = [0; BUFFER_SIZE];
    let mut bytes_copied = 0u64;

    loop {
        let bytes_read = source_file
            .read(&mut buffer)
            .map_err(|e| OrganizeError::FileCopyFailed(e.to_string()))?;

        if bytes_read == 0 {
            // Make sure to call progress one last time with total size
            progress_callback(file_size);
            break;
        }

        target_file
            .write_all(&buffer[..bytes_read])
            .map_err(|e| OrganizeError::FileCopyFailed(e.to_string()))?;

        bytes_copied += bytes_read as u64;
        progress_callback(bytes_copied);
    }

    Ok(())
}
