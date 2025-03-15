use crate::error::OrganizeError;
use crate::models::{CustomFile, SaveState};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const BUFFER_SIZE: usize = 8192;

pub fn organize_files<F>(
    files: Vec<CustomFile>,
    output_path: &PathBuf,
    mut progress_callback: F,
    stop_signal: Arc<AtomicBool>,
) -> Result<Option<SaveState>, OrganizeError>
where
    F: FnMut(&str, u64, u64, usize),
{
    let mut save_state = SaveState::new(
        files
            .first()
            .map(|f| f.path.parent().unwrap_or(&f.path).to_path_buf())
            .unwrap_or_default(),
        output_path.clone(),
    );

    for (index, file) in files.into_iter().enumerate() {
        if stop_signal.load(Ordering::Relaxed) {
            return Ok(Some(save_state));
        }

        let file_type = file.get_type();
        let date = file
            .get_creation_date()
            .map_err(|e| OrganizeError::UserInputError(e))?;

        let type_dir = output_path.join(format!("{:?}", file_type));
        let date_dir = type_dir.join(date);

        fs::create_dir_all(&date_dir)
            .map_err(|e| OrganizeError::DirectoryCreationFailed(e.to_string()))?;

        let target_path = date_dir.join(&file.name);

        // Copy file with progress
        copy_file_with_progress(
            &file.path,
            &target_path,
            &file.name,
            file.meta.len(),
            |bytes_copied| {
                progress_callback(&file.name, file.meta.len(), bytes_copied, index + 1);
            },
        )?;

        // Add to save state after successful copy
        save_state.add_processed_file(
            file.path.clone(),
            file.name.clone(),
            file.meta.len(),
            file.meta
                .modified()
                .map_err(|e| OrganizeError::UserInputError(e.to_string()))?,
        );
    }

    Ok(None) // None means all files were processed successfully
}

fn copy_file_with_progress<F>(
    source: &PathBuf,
    target: &PathBuf,
    file_name: &str,
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
