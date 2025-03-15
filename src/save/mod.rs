use colored::*;
use std::path::PathBuf;
use crate::models::SaveState;
use crate::error::OrganizeError;

pub fn save_progress(save_state: SaveState) -> Result<(), OrganizeError> {
    let save_path = save_state
        .save()
        .map_err(|e| OrganizeError::FileCopyFailed(format!("Failed to save progress: {}", e)))?;

    println!(
        "\n{} {}",
        "📝 Progress saved at:".bright_yellow(),
        save_path.display()
    );
    Ok(())
}

pub fn handle_save_cleanup(resume_path: Option<PathBuf>) {
    if let Some(path) = resume_path {
        if let Err(e) = std::fs::remove_file(path) {
            eprintln!("{} {}", "Failed to clean up save file:".yellow(), e);
        }
    }
}