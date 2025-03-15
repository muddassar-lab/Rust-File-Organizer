use std::sync::{atomic::{AtomicBool, Ordering}, Arc, mpsc};
use indicatif::ProgressBar;
use crate::{
    models::CustomFile,
    organizer::organize_files,
    ui::{update_total_progress, update_file_progress},
    OrganizeError,
};

pub fn spawn_processing_thread(
    files: Vec<CustomFile>,
    output_path: std::path::PathBuf,
    total_progress: ProgressBar,
    file_progress: ProgressBar,
    stop_signal: Arc<AtomicBool>,
    total_files: u64,
    tx: mpsc::Sender<()>,
) -> std::thread::JoinHandle<Result<Option<crate::models::SaveState>, OrganizeError>> {
    std::thread::spawn({
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

            let _ = tx.send(());
            total_progress.finish_and_clear();
            file_progress.finish_and_clear();
            result
        }
    })
}