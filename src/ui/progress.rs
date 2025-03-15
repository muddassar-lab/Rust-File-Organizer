use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

pub fn format_size(size: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{:.0} {}", size, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

pub fn create_progress_bars() -> (MultiProgress, ProgressBar, ProgressBar) {
    let multi = MultiProgress::new();
    let total_progress = ProgressBar::new(0);
    let file_progress = ProgressBar::new(0);

    total_progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [Total time: {elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files ({eta} remaining)")
            .unwrap()
            .progress_chars("█▇▆▅▄▃▂▁  "),
    );

    file_progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta} remaining)\n🗂️  {prefix:.green}\n⏳ {msg}")
            .unwrap()
            .progress_chars("█▇▆▅▄▃▂▁  "),
    );

    // Increase tick rate for smoother updates
    file_progress.enable_steady_tick(Duration::from_millis(50));
    total_progress.enable_steady_tick(Duration::from_millis(50));

    (multi, total_progress, file_progress)
}

pub fn update_total_progress(progress_bar: &ProgressBar, total: u64, current: u64) {
    progress_bar.set_length(total);
    progress_bar.set_position(current);
}
pub fn update_file_progress(
    progress_bar: &ProgressBar,
    file_name: &str,
    file_size: u64,
    bytes_copied: u64,
) {
    progress_bar.set_length(file_size);
    progress_bar.set_position(bytes_copied);
    progress_bar.set_prefix(file_name.to_string());
    progress_bar.set_message(format!("Total size: {}", format_size(file_size)));
}
