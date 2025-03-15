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
    path::PathBuf,
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

fn select_operation_mode() -> Option<PathBuf> {
    println!("\n{}", "Select operation mode:".bright_cyan());

    let options = vec!["Start new organization", "Resume from saved file"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&options)
        .default(0)
        .interact()
        .unwrap_or(0);

    if selection == 1 {
        match SaveState::list_saves() {
            Ok(saves) if !saves.is_empty() => {
                let save_options: Vec<String> = saves
                    .iter()
                    .map(|path| {
                        path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                    })
                    .collect();

                let save_selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select a save file:")
                    .items(&save_options)
                    .default(0)
                    .interact()
                    .ok()
                    .map(|i| saves[i].clone());

                save_selection
            }
            _ => {
                println!("{}", "No save files found.".yellow());
                None
            }
        }
    } else {
        None
    }
}

fn main() {
    print_header();

    // Create a stop signal that will be shared between threads
    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_signal_clone = Arc::clone(&stop_signal);

    // Set up the Ctrl+C handler before any other operations
    ctrlc::set_handler(move || {
        // Don't print anything here as it will mess up the progress bars
        stop_signal_clone.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    // Add a variable to track if we're resuming and store the save path
    let (input_path, output_path, files, resume_path, save_state) = match select_operation_mode() {
        Some(save_path) => match SaveState::load(&save_path) {
            Ok(save_state) => {
                println!("{}", "📝 Resuming from save file...".bright_green());
                println!(
                    "{} {}",
                    "Input folder:".green(),
                    save_state.input_path.display()
                );
                println!(
                    "{} {}",
                    "Output folder:".green(),
                    save_state.output_path.display()
                );

                let processed_paths: std::collections::HashSet<_> = save_state
                    .processed_files
                    .iter()
                    .map(|f| f.path.clone())
                    .collect();

                let all_files = get_all_files(&save_state.input_path);
                let remaining_files: Vec<_> = all_files
                    .into_iter()
                    .filter(|f| !processed_paths.contains(&f.path))
                    .collect();

                (
                    save_state.input_path.clone(),
                    save_state.output_path.clone(),
                    remaining_files,
                    Some(save_path),
                    Some(save_state),
                )
            }
            Err(e) => {
                eprintln!("{} {}", "Failed to load save file:".red(), e);
                process::exit(1);
            }
        },
        None => {
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

            println!("\n{}", "🔍 Scanning files...".bright_cyan());
            let files = get_all_files(&input_path);
            (input_path, output_path, files, None, None) // Add None for save_state
        }
    };

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

    // If resuming, update the initial progress to include already processed files
    if let Some(ref save_state) = save_state {
        let processed_count = save_state.processed_files.len() as u64;
        update_total_progress(
            &total_progress,
            total_files + processed_count,
            processed_count,
        );
    }

    // Create a channel to signal when the file copy is complete
    let (tx, rx) = std::sync::mpsc::channel();

    let result = std::thread::spawn({
        let total_progress = total_progress.clone();
        let file_progress = file_progress.clone();
        let output_path = output_path.clone();
        let stop_signal = Arc::clone(&stop_signal);
        let tx = tx.clone();
        move || {
            let result = organize_files(
                files,
                &output_path,
                |file_name, file_size, bytes_copied, current_file| {
                    update_total_progress(&total_progress, total_files, current_file as u64);
                    update_file_progress(
                        &file_progress,
                        file_name,
                        file_size,
                        bytes_copied,
                        stop_signal.load(Ordering::SeqCst),
                    );
                },
                Arc::clone(&stop_signal),
            );

            // Signal that we're done with file operations
            let _ = tx.send(());

            // Clear progress bars only after we're completely done
            total_progress.finish_and_clear();
            file_progress.finish_and_clear();

            result
        }
    });

    // Wait for either the thread to complete or the stop signal
    if stop_signal.load(Ordering::SeqCst) {
        // Wait for the current file to complete
        let _ = rx.recv();
        println!("\n{}", "🛑 Process interrupted!".yellow());
    }

    // Wait for the thread to complete
    let result = match result.join() {
        Ok(r) => r,
        Err(_) => {
            handle_error("Thread panicked", Some(&output_path));
            return;
        }
    };

    match result {
        Ok(Some(mut save_state)) => {
            println!("\n{}", "🛑 Process interrupted!".yellow());

            match resume_path {
                // If we have a resume path, merge with existing save state and update the file
                Some(path) => {
                    if let Ok(mut existing_save) = SaveState::load(&path) {
                        // Merge the newly processed files with existing ones
                        existing_save
                            .processed_files
                            .extend(save_state.processed_files);
                        save_state = existing_save;
                    }

                    if let Err(e) =
                        std::fs::write(&path, serde_json::to_string_pretty(&save_state).unwrap())
                    {
                        handle_error(
                            OrganizeError::FileCopyFailed(format!(
                                "Failed to update save file: {}",
                                e
                            )),
                            Some(&output_path),
                        );
                    }
                    println!(
                        "\n{}",
                        "📝 Progress automatically saved to existing file.".bright_yellow()
                    );
                    println!("\n{}", "👋 Goodbye!".bright_blue());
                }
                // For new sessions, ask the user what to do
                None => {
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

            // Clean up the save file if it exists and the process completed
            if let Some(path) = resume_path {
                if let Err(e) = std::fs::remove_file(path) {
                    eprintln!("{} {}", "Failed to clean up save file:".yellow(), e);
                }
            }
        }
        Err(e) => handle_error(e, Some(&output_path)),
    }
}
