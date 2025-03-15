use chrono::{DateTime, Local};
use colored::*;
use dialoguer::Confirm;
use dirs::download_dir;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use rfd::FileDialog;
use std::{
    fs::{self, Metadata},
    path::PathBuf,
    process,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

/// Represents input and output paths for file organization
#[derive(Debug)]
struct Paths {
    input_path: PathBuf,
    output_path: PathBuf,
}

/// Represents a file with its metadata and path information
#[derive(Debug)]
struct CustomFile {
    extension: String,
    name: String,
    path: PathBuf,
    meta: Metadata,
}

/// Represents different types of files that can be organized
#[derive(Debug)]
enum FileType {
    Video,
    Music,
    Document,
    Picture,
    Program,
    Other,
}

impl FileType {
    /// Returns the appropriate FileType based on file extension
    fn from_extension(extension: &str) -> Self {
        match extension.to_lowercase().as_str() {
            "mp4" | "mkv" | "avi" | "mov" | "wmv" => FileType::Video,
            "mp3" | "wav" | "flac" | "m4a" | "ogg" => FileType::Music,
            "pdf" | "doc" | "docx" | "txt" | "rtf" => FileType::Document,
            "jpg" | "jpeg" | "png" | "gif" | "bmp" => FileType::Picture,
            "exe" | "msi" | "bat" | "sh" | "app" => FileType::Program,
            _ => FileType::Other,
        }
    }
}

impl CustomFile {
    /// Creates a new CustomFile from a path
    ///
    /// # Arguments
    /// * `path` - The path to the file
    ///
    /// # Returns
    /// * `Option<Self>` - A new CustomFile if all components are valid
    fn from_path(path: &PathBuf) -> Option<Self> {
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

    /// Returns the FileType of the current file
    fn get_type(&self) -> FileType {
        FileType::from_extension(&self.extension)
    }

    /// Gets the creation date of the file in YYYY-MM-DD format
    fn get_creation_date(&self) -> Result<String, String> {
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

/// Error type for file organization operations
#[derive(Debug)]
enum OrganizeError {
    NoPathSelected,
    InvalidFolderName,
    NoParentDirectory,
    DirectoryCreationFailed(String),
    FileCopyFailed(String),
    UserInputError(String),
    InvalidOutputPath(String),
}

impl std::fmt::Display for OrganizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoPathSelected => write!(f, "No folder selected"),
            Self::InvalidFolderName => write!(f, "Invalid folder name"),
            Self::NoParentDirectory => write!(f, "Could not determine parent directory"),
            Self::DirectoryCreationFailed(e) => write!(f, "Failed to create directory: {}", e),
            Self::FileCopyFailed(e) => write!(f, "Failed to copy file: {}", e),
            Self::UserInputError(e) => write!(f, "User input error: {}", e),
            Self::InvalidOutputPath(e) => write!(f, "Invalid output path: {}", e),
        }
    }
}

/// Gets input and output paths for file organization
fn get_paths() -> Result<Paths, OrganizeError> {
    println!("{}", "📂 Selecting directories...".bright_blue());

    let start_dir = download_dir().unwrap_or_else(|| {
        println!(
            "{}",
            "⚠️  Could not find download directory, using current directory".yellow()
        );
        std::env::current_dir().expect("Failed to get current directory")
    });

    let input_path = FileDialog::new()
        .set_directory(&start_dir)
        .pick_folder()
        .ok_or(OrganizeError::NoPathSelected)?;

    println!(
        "{} {}",
        "📁 Selected input directory:".green(),
        input_path.display().to_string().bright_white()
    );

    // Ask if user wants to select output directory
    let custom_output = Confirm::new()
        .with_prompt("Would you like to select a custom output directory?")
        .default(false)
        .interact()
        .map_err(|_| OrganizeError::UserInputError("Failed to get user input".to_string()))?;

    let output_path = if custom_output {
        let output = FileDialog::new()
            .set_directory(&start_dir)
            .pick_folder()
            .ok_or(OrganizeError::NoPathSelected)?;

        // Check if output directory is same as input
        if output == input_path {
            return Err(OrganizeError::InvalidOutputPath(
                "Output directory cannot be the same as input directory".to_string(),
            ));
        }

        // Create a subdirectory in the selected output directory
        let output_folder_name = input_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| format!("{} Organized", name))
            .ok_or(OrganizeError::InvalidFolderName)?;

        let final_output = output.join(output_folder_name);

        println!(
            "{} {}",
            "📁 Selected output directory:".green(),
            final_output.display().to_string().bright_white()
        );

        final_output
    } else {
        // Default behavior: create output directory next to input directory
        let output_folder_name = input_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| format!("{} Organized", name))
            .ok_or(OrganizeError::InvalidFolderName)?;

        let parent_path = input_path
            .parent()
            .ok_or(OrganizeError::NoParentDirectory)?;

        let output = parent_path.join(&output_folder_name);
        println!(
            "{} {}",
            "📁 Output directory:".green(),
            output.display().to_string().bright_white()
        );

        output
    };

    // Recreate output directory if it exists
    if output_path.exists() {
        println!(
            "{}",
            "🗑️  Cleaning up existing output directory...".yellow()
        );
        fs::remove_dir_all(&output_path)
            .map_err(|e| OrganizeError::DirectoryCreationFailed(e.to_string()))?;
    }

    fs::create_dir(&output_path)
        .map_err(|e| OrganizeError::DirectoryCreationFailed(e.to_string()))?;

    Ok(Paths {
        input_path,
        output_path,
    })
}

/// Recursively collects all files from a directory
fn get_all_files(input_path: &PathBuf) -> Vec<CustomFile> {
    println!("{}", "\n🔍 Scanning files...".bright_blue());
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.blue} [{elapsed_precise}] {msg}")
            .unwrap(),
    );

    let files = Arc::new(Mutex::new(Vec::new()));
    let count = Arc::new(Mutex::new(0));

    // Function to update progress
    let update_progress = |current_path: &PathBuf, files_count: usize| {
        spinner.set_message(format!(
            "Found {} files | Scanning: {}",
            files_count,
            current_path.display().to_string().bright_white()
        ));
    };

    // Recursive function to scan directories
    fn scan_directory(
        path: &PathBuf,
        files: &Arc<Mutex<Vec<CustomFile>>>,
        count: &Arc<Mutex<usize>>,
        update_fn: &(dyn Fn(&PathBuf, usize) + Send + Sync),
    ) {
        if let Ok(entries) = fs::read_dir(path) {
            // Convert entries to a vector for parallel processing
            let entries: Vec<_> = entries.filter_map(Result::ok).collect();

            entries.par_iter().for_each(|entry| {
                let path = entry.path();

                if path.is_file() {
                    if let Some(file) = CustomFile::from_path(&path) {
                        let mut files = files.lock().unwrap();
                        files.push(file);
                        let new_count = files.len();
                        drop(files); // Release the lock
                        *count.lock().unwrap() = new_count;
                        (update_fn)(&path, new_count); // Fix function call syntax
                    }
                } else if path.is_dir() {
                    scan_directory(&path, files, count, update_fn);
                }
            });
        }
    }

    // Start the scanning process
    scan_directory(input_path, &files, &count, &update_progress);

    let final_files = Arc::try_unwrap(files)
        .expect("Failed to unwrap Arc")
        .into_inner()
        .expect("Failed to unwrap Mutex");

    spinner.finish_with_message(
        format!(
            "✓ Found {} files in {}",
            final_files.len(),
            input_path.display().to_string()
        )
        .bright_green()
        .to_string(),
    );

    final_files
}

/// Organizes files by type and creation date
fn organize_files(
    files: Vec<CustomFile>,
    output_path: &PathBuf,
) -> Result<Vec<PathBuf>, OrganizeError> {
    let mut organized_files = Vec::new();

    println!("{}", "\n📦 Organizing files...".bright_blue());
    let multi = MultiProgress::new();
    let total_pb = multi.add(ProgressBar::new(files.len() as u64));
    let file_pb = multi.add(ProgressBar::new(1));

    total_pb.set_style(
        ProgressStyle::default_bar()
            .template("{prefix:.bold.dim} {bar:40.cyan/blue} {pos}/{len} ({percent}%) {msg}")
            .unwrap()
            .progress_chars("█▇▆▅▄▃▂▁  "),
    );
    total_pb.set_prefix("Total progress:");

    file_pb.set_style(
        ProgressStyle::default_bar()
            .template("{prefix:.bold.dim} {msg}")
            .unwrap(),
    );
    file_pb.set_prefix("Current file:");

    for (i, file) in files.iter().enumerate() {
        let file_type = file.get_type();
        let type_dir = output_path.join(format!("{:?}", file_type));

        // Create type directory
        if !type_dir.exists() {
            fs::create_dir(&type_dir)
                .map_err(|e| OrganizeError::DirectoryCreationFailed(e.to_string()))?;
        }

        // Create date directory
        let date_dir = type_dir.join(
            file.get_creation_date()
                .map_err(|e| OrganizeError::DirectoryCreationFailed(e))?,
        );

        if !date_dir.exists() {
            fs::create_dir(&date_dir)
                .map_err(|e| OrganizeError::DirectoryCreationFailed(e.to_string()))?;
        }

        // Copy file
        let dest_path = date_dir.join(&file.name);
        file_pb.set_message(format!("Copying: {}", file.name.bright_white()));

        fs::copy(&file.path, &dest_path).map_err(|e| {
            OrganizeError::FileCopyFailed(format!(
                "Failed to copy '{}' to '{}': {}",
                file.path.display(),
                dest_path.display(),
                e
            ))
        })?;

        organized_files.push(dest_path);
        total_pb.inc(1);

        // Add a small delay to make the progress visible
        thread::sleep(Duration::from_millis(50));
    }

    total_pb.finish_with_message("Complete!".bright_green().to_string());
    file_pb.finish_and_clear();

    Ok(organized_files)
}

/// Removes the output directory and its contents
fn cleanup(output_dir: &PathBuf) {
    println!("{}", "🗑️  Cleaning up...".yellow());
    if let Err(e) = fs::remove_dir_all(output_dir) {
        eprintln!(
            "{} {}",
            "Failed to clean up directory:".red(),
            e.to_string().bright_red()
        );
    }
}

fn main() {
    println!("{}", "\n🚀 File Organizer Starting...".bright_blue());

    let paths = match get_paths() {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!(
                "{} {}",
                "Error setting up paths:".red(),
                e.to_string().bright_red()
            );
            process::exit(1);
        }
    };

    let files = get_all_files(&paths.input_path);
    if files.is_empty() {
        eprintln!("{}", "No files found in the selected directory".yellow());
        cleanup(&paths.output_path);
        process::exit(1);
    }

    match organize_files(files, &paths.output_path) {
        Ok(_) => println!("\n{}", "✨ Files organized successfully!".bright_green()),
        Err(e) => {
            eprintln!(
                "{} {}",
                "Error organizing files:".red(),
                e.to_string().bright_red()
            );
            cleanup(&paths.output_path);
            process::exit(1);
        }
    }
}
