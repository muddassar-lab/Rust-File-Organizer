use colored::*;
use dialoguer::{Select, theme::ColorfulTheme};
use file_organizer::{
    OrganizeError,
    models::SaveState,
    organizer::{get_all_files, organize_files},
    ui::{
        cleanup, create_progress_bars, get_output_choice, get_output_location,
        update_file_progress, update_total_progress,
    },
};
use std::{
    process,
    sync::Arc,
    sync::atomic::{AtomicBool, Ordering},
};

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

fn save_progress(save_state: SaveState) -> Result<(), OrganizeError> {
    let save_path = save_state.output_path.with_extension("forg");
    save_state
        .save_to_file(&save_path)
        .map_err(|e| OrganizeError::FileCopyFailed(format!("Failed to save progress: {}", e)))?;

    println!(
        "\n{} {}",
        "📝 Progress saved at:".bright_yellow(),
        save_path.display()
    );
    Ok(())
}

fn main() {
    print_header();

    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_signal_clone = stop_signal.clone();

    // Set up Ctrl+C handler
    ctrlc::set_handler(move || {
        stop_signal_clone.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl+C handler");

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
    println!("Press Ctrl+C to stop the process");

    let (multi, total_progress, file_progress) = create_progress_bars();
    let total_files = files.len() as u64;

    let total_progress = multi.add(total_progress);
    let file_progress = multi.add(file_progress);

    let result = std::thread::spawn({
        let total_progress = total_progress.clone();
        let file_progress = file_progress.clone();
        let output_path = output_path.clone();
        let stop_signal = stop_signal.clone();
        move || {
            organize_files(
                files,
                &output_path,
                |file_name, file_size, bytes_copied, current_file| {
                    update_total_progress(&total_progress, total_files, current_file as u64);
                    update_file_progress(&file_progress, file_name, file_size, bytes_copied);
                },
                stop_signal,
            )
        }
    })
    .join()
    .unwrap();

    total_progress.finish_and_clear();
    file_progress.finish_and_clear();

    match result {
        Ok(Some(save_state)) => {
            println!("\n{}", "🛑 Process interrupted!".yellow());

            let options = vec!["Save progress and exit", "Just exit"];
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("What would you like to do?")
                .items(&options)
                .default(0)
                .interact()
                .unwrap_or(1);

            match selection {
                0 => {
                    if let Err(e) = save_progress(save_state) {
                        handle_error(e, Some(&output_path));
                    }
                    println!("\n{}", "👋 Goodbye!".bright_blue());
                }
                _ => {
                    cleanup(&output_path);
                    println!("\n{}", "👋 Goodbye!".bright_blue());
                }
            }
        }
        Ok(None) => {
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
