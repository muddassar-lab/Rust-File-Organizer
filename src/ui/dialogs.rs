use crate::error::OrganizeError;
use crate::models::Paths;
use rfd::FileDialog;

pub fn get_output_location() -> Result<Paths, OrganizeError> {
    let input_path = FileDialog::new()
        .set_title("Select folder to organize")
        .set_directory(dirs::home_dir().unwrap_or_default()) // Start from home directory
        .pick_folder()
        .ok_or(OrganizeError::NoPathSelected)?;

    Ok(Paths {
        output_path: input_path.clone(), // This will be replaced by main.rs logic
        input_path,
    })
}
