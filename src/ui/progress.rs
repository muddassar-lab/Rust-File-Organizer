use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub fn create_progress_bars() -> (MultiProgress, ProgressBar) {
    let multi = MultiProgress::new();
    let progress_bar = ProgressBar::new(0);
    
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    (multi, progress_bar)
}

pub fn update_progress(progress_bar: &ProgressBar, total: u64, current: u64) {
    progress_bar.set_length(total);
    progress_bar.set_position(current);
}