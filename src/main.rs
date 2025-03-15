use colored::*;
use file_organizer::{
    organizer::{get_all_files, organize_files},
    ui::{cleanup, create_progress_bars, get_output_choice, get_output_location, update_progress},
};
use std::process;

fn print_header() {
    println!("{}", "\n🚀 File Organizer v1.0".bright_blue().bold());
    println!("{}", "===================".bright_blue());
}

fn handle_error(error: impl std::fmt::Display, output_path: Option<&std::path::PathBuf>) {
    if let Some(path) = output_path {
        cleanup(path);
    }
    eprintln!("{} {}", "❌ Error:".red(), error.to_string().bright_red());
    process::exit(1);
}

fn main() {
    print_header();

    // Get input and output paths
    let input_path = get_output_location()
        .map(|paths| paths.input_path)
        .expect("Failed to get input location");

    println!(
        "{} {}",
        "Selected input folder:".green(),
        input_path.display()
    );

    let output_path = get_output_choice(&input_path);
    println!(
        "{} {}",
        "Selected output folder:".green(),
        output_path.display()
    );

    // Scan files
    println!("\n{}", "🔍 Scanning files...".bright_cyan());
    let files = get_all_files(&input_path);
    if files.is_empty() {
        handle_error(
            "No files found in the selected directory",
            Some(&output_path),
        );
    }

    println!(
        "{} {} {}",
        "Found".green(),
        files.len().to_string().bright_green(),
        "files".green()
    );

    // Process files
    println!("\n{}", "📊 Organizing files...".bright_cyan());
    let (_, progress_bar) = create_progress_bars();
    let total_files = files.len() as u64;

    let result = std::thread::spawn({
        let progress_bar = progress_bar.clone();
        let output_path = output_path.clone();
        move || {
            organize_files(files, &output_path, |current| {
                update_progress(&progress_bar, total_files, current as u64);
            })
        }
    })
    .join()
    .unwrap();

    // Handle result
    match result {
        Ok(_) => {
            println!(
                "\n{}",
                "✨ Organization completed successfully!"
                    .bright_green()
                    .bold()
            );
            println!(
                "{} {}",
                "📍 Files organized at:".bright_cyan(),
                output_path.display()
            );
        }
        Err(e) => handle_error(e, Some(&output_path)),
    }
}
