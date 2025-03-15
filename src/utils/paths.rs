use std::fs;
use std::path::PathBuf;

pub fn get_save_dir() -> PathBuf {
    let mut app_data = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("file-organizer");

    // Create the directory if it doesn't exist
    if !app_data.exists() {
        fs::create_dir_all(&app_data).expect("Failed to create application data directory");
    }

    app_data.join("saves")
}

pub fn ensure_save_dir() -> std::io::Result<()> {
    let save_dir = get_save_dir();
    if !save_dir.exists() {
        fs::create_dir_all(save_dir)?;
    }
    Ok(())
}

pub fn generate_save_filename(input_path: &PathBuf) -> String {
    use chrono::Local;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let folder_name = input_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();

    format!("{}_{}.forg", folder_name, timestamp)
}
