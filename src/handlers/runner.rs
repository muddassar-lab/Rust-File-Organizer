use colored::*;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use crate::{
    cli::{handle_error, print_header, select_operation_mode},
    handlers::{InitResult, handle_organization_result, initialize_app, spawn_processing_thread},
    ui::{create_progress_bars, update_total_progress},
};

pub fn run_app() {
    print_header();

    // Create a stop signal that will be shared between threads
    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_signal_clone = Arc::clone(&stop_signal);

    // Set up the Ctrl+C handler before any other operations
    ctrlc::set_handler(move || {
        stop_signal_clone.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    let InitResult {
        input_path,
        output_path,
        files,
        resume_path,
        save_state,
    } = initialize_app(select_operation_mode());

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

    if let Some(ref save_state) = save_state {
        let processed_count = save_state.processed_files.len() as u64;
        update_total_progress(
            &total_progress,
            total_files + processed_count,
            processed_count,
        );
    }

    let (tx, rx) = std::sync::mpsc::channel();

    let result = spawn_processing_thread(
        files,
        output_path.clone(),
        total_progress,
        file_progress,
        Arc::clone(&stop_signal),
        total_files,
        tx,
    );

    if stop_signal.load(Ordering::SeqCst) {
        let _ = rx.recv();
        println!("\n{}", "🛑 Process interrupted!".yellow());
    }

    let result = match result.join() {
        Ok(r) => r,
        Err(_) => {
            handle_error("Thread panicked", Some(&output_path));
            return;
        }
    };

    handle_organization_result(result, resume_path, output_path);
}