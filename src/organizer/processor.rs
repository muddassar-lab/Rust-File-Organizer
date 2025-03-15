use crate::error::OrganizeError;
use crate::models::CustomFile;
use std::fs;
use std::path::PathBuf;

pub fn organize_files<F>(
    files: Vec<CustomFile>,
    output_path: &PathBuf,
    progress_callback: F,
) -> Result<(), OrganizeError>
where
    F: Fn(usize),
{
    for (index, file) in files.into_iter().enumerate() {
        let file_type = file.get_type();
        let date = file
            .get_creation_date()
            .map_err(|e| OrganizeError::UserInputError(e))?;

        let type_dir = output_path.join(format!("{:?}", file_type));
        let date_dir = type_dir.join(date);

        fs::create_dir_all(&date_dir)
            .map_err(|e| OrganizeError::DirectoryCreationFailed(e.to_string()))?;

        let target_path = date_dir.join(&file.name);
        fs::copy(&file.path, &target_path)
            .map_err(|e| OrganizeError::FileCopyFailed(e.to_string()))?;

        progress_callback(index + 1);
    }

    Ok(())
}
