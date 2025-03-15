use crate::{
    OrganizeError,
    models::CustomFile,
    organizer::organize_files,
    ui::progress::{ProgressUI, ProgressUpdate},
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::time::Instant;

pub fn spawn_processing_thread(
    files: Vec<CustomFile>,
    output_path: std::path::PathBuf,
    stop_signal: Arc<AtomicBool>,
    total_files: u64,
) -> (
    std::thread::JoinHandle<Result<Option<crate::models::SaveState>, OrganizeError>>,
    mpsc::Receiver<ProgressUpdate>,
) {
    let (tx, rx) = mpsc::channel();
    let handle = std::thread::spawn({
        let tx = tx.clone();
        let output_path = output_path.clone();
        let stop_signal = Arc::clone(&stop_signal);
        move || {
            let last_update = Arc::new(std::sync::Mutex::new((Instant::now(), 0u64)));

            let result = organize_files(
                files,
                &output_path,
                |file_name, file_size, bytes_copied, current_file| {
                    let mut last = last_update.lock().unwrap();
                    let now = Instant::now();
                    let elapsed = now.duration_since(last.0);

                    if elapsed.as_millis() >= 50 {
                        // Calculate bytes processed since last update, handling potential overflow
                        let bytes_since_last = if bytes_copied >= last.1 {
                            bytes_copied - last.1
                        } else {
                            // If we're processing a new file, just use the current bytes_copied
                            bytes_copied
                        };

                        let bytes_per_second = if elapsed.as_secs_f64() > 0.0 {
                            bytes_since_last as f64 / elapsed.as_secs_f64()
                        } else {
                            0.0
                        };

                        let _ = tx.send(ProgressUpdate::File {
                            name: file_name.to_string(),
                            size: file_size,
                            progress: bytes_copied,
                            index: current_file as u64,
                            bytes_per_second,
                            total_bytes: bytes_copied,
                            estimated_time: if bytes_per_second > 0.0 {
                                Some((file_size - bytes_copied) as f64 / bytes_per_second)
                            } else {
                                None
                            },
                        });

                        // Update last values
                        last.0 = now;
                        last.1 = bytes_copied;
                    }
                },
                Arc::clone(&stop_signal),
            );

            let _ = tx.send(ProgressUpdate::Complete);
            result
        }
    });

    (handle, rx)
}
