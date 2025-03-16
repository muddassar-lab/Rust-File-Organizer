use crate::{
    cli::{handle_error, print_header, select_operation_mode},
    handlers::{InitResult, handle_organization_result, initialize_app, spawn_processing_thread},
    ui::progress::ProgressUI,
};
use colored::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub fn run_app() {
    print_header();

    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_signal_clone = Arc::clone(&stop_signal);

    ctrlc::set_handler(move || {
        stop_signal_clone.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    let InitResult {
        input_path: _,
        output_path,
        files,
        resume_path,
        save_state: _,
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

    println!("\n{}", "📊 Organizing files...".bright_cyan());
    println!("Press 'q' to quit or Ctrl+C to stop the process");

    let total_files = files.len() as u64;
    let mut ui = ProgressUI::new(total_files).expect("Failed to create UI");

    // Initialize the file queue with all files
    let file_queue: Vec<(String, u64)> = files
        .iter()
        .map(|f| (f.name.clone(), f.meta.len()))
        .collect();
    ui.set_file_queue(file_queue);

    let (handle, rx) = spawn_processing_thread(
        files,
        output_path.clone(),
        Arc::clone(&stop_signal),
        total_files,
    );

    if let Err(e) = ui.run(rx) {
        handle_error(format!("UI error: {}", e), Some(&output_path));
        return;
    }

    let result = match handle.join() {
        Ok(r) => r,
        Err(_) => {
            handle_error("Thread panicked", Some(&output_path));
            return;
        }
    };

    handle_organization_result(result, resume_path, output_path);
}
