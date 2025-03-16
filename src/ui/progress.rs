use crate::models::FileType;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};
use std::{
    io,
    sync::mpsc,
    time::{Duration, Instant},
};

pub struct ProgressState {
    pub total_files: u64,
    pub current_file_index: u64,
    pub current_file: String,
    pub current_file_size: u64,
    pub current_file_progress: u64,
    pub start_time: Instant,
    pub bytes_per_second: f64,
    pub is_stopping: bool,
    pub recent_files: Vec<String>,
    pub total_bytes: u64,
    pub estimated_time: Option<f64>,
    pub file_queue: Vec<(String, u64)>, // (filename, size) pairs for upcoming files
    pub last_file: Option<String>,
}

impl ProgressState {
    pub fn new(total_files: u64) -> Self {
        ProgressState {
            total_files,
            current_file_index: 0,
            current_file: String::new(),
            current_file_size: 0,
            current_file_progress: 0,
            start_time: Instant::now(),
            bytes_per_second: 0.0,
            is_stopping: false,
            recent_files: Vec::new(),
            total_bytes: 0,
            estimated_time: None,
            file_queue: Vec::new(),
            last_file: None,
        }
    }

    pub fn set_file_queue(&mut self, files: Vec<(String, u64)>) {
        self.file_queue = files;
    }

    pub fn update_file_progress(
        &mut self,
        file_name: String,
        size: u64,
        progress: u64,
        index: u64,
        bps: f64,
        total_bytes: u64,
        estimated_time: Option<f64>,
    ) {
        self.current_file = file_name.clone();
        self.current_file_size = size;
        self.current_file_progress = progress;
        self.current_file_index = index;
        self.bytes_per_second = bps;
        self.total_bytes = total_bytes;
        self.estimated_time = estimated_time;
        if self.last_file.is_some() && self.last_file != Some(file_name.clone()) {
            self.recent_files.push(self.last_file.take().unwrap());
        }
        self.last_file = Some(file_name);
    }
}

pub struct ProgressUI {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    state: ProgressState,
}

impl ProgressUI {
    pub fn new(total_files: u64) -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            state: ProgressState::new(total_files),
        })
    }

    pub fn run(&mut self, rx: mpsc::Receiver<ProgressUpdate>) -> io::Result<()> {
        loop {
            let ui_state = &self.state;
            self.terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints(
                        [
                            Constraint::Length(3), // Total progress
                            Constraint::Length(3), // File progress
                            Constraint::Min(6),    // Files and Stats section
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                Self::render_total_progress(ui_state, f, chunks[0]);
                Self::render_file_progress(ui_state, f, chunks[1]);
                Self::render_recent_files(ui_state, f, chunks[2]);
            })?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') {
                        return Ok(());
                    }
                }
            }

            if let Ok(update) = rx.try_recv() {
                match update {
                    ProgressUpdate::File {
                        name,
                        size,
                        progress,
                        index,
                        bytes_per_second,
                        total_bytes,
                        estimated_time,
                    } => {
                        self.state.update_file_progress(
                            name,
                            size,
                            progress,
                            index,
                            bytes_per_second,
                            total_bytes,
                            estimated_time,
                        );
                    }
                    ProgressUpdate::Stop => {
                        self.state.is_stopping = true;
                    }
                    ProgressUpdate::Complete => {
                        return Ok(());
                    }
                }
            }
        }
    }

    pub fn set_file_queue(&mut self, files: Vec<(String, u64)>) {
        self.state.set_file_queue(files);
    }

    fn render_total_progress(state: &ProgressState, f: &mut Frame, area: Rect) {
        let ratio = state.current_file_index as f64 / state.total_files as f64;
        let percentage = (ratio * 100.0) as u64;

        // Calculate processed and remaining files
        let files_remaining = state.total_files - state.current_file_index;
        let processed_bytes = format_size(state.total_bytes);

        // Calculate average speed and estimated total time
        let elapsed = state.start_time.elapsed().as_secs_f64();
        let avg_speed = if elapsed > 0.0 {
            state.total_bytes as f64 / elapsed
        } else {
            0.0
        };

        // Format the progress details
        let progress_text = format!(
            "{}/{} files ({}%) | Processed: {} | Avg Speed: {}/s | Remaining: {} files",
            state.current_file_index,
            state.total_files,
            percentage,
            processed_bytes,
            format_size(avg_speed as u64),
            files_remaining
        );

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(Span::styled(
                        "Total Progress",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL),
            )
            .gauge_style(
                Style::default()
                    .fg(Color::Cyan)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .ratio(ratio)
            .label(progress_text);
        f.render_widget(gauge, area);
    }

    fn render_file_progress(state: &ProgressState, f: &mut Frame, area: Rect) {
        let ratio = if state.current_file_size > 0 {
            state.current_file_progress as f64 / state.current_file_size as f64
        } else {
            0.0
        };

        let title = if state.is_stopping {
            "Current File (Stopping...)"
        } else {
            "Current File"
        };

        // Get file extension and determine file type
        let file_ext = std::path::Path::new(&state.current_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        let file_type = FileType::from_extension(file_ext);

        // Format progress details
        let progress_text = format!(
            "{} [{:?}] {}/{} ({}%)",
            state.current_file,
            file_type,
            format_size(state.current_file_progress),
            format_size(state.current_file_size),
            ((ratio * 100.0) as u64)
        );

        let gauge = Gauge::default()
            .block(Block::default().title(title).borders(Borders::ALL))
            .gauge_style(
                Style::default()
                    .fg(Color::Green)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .ratio(ratio)
            .label(progress_text);
        f.render_widget(gauge, area);
    }

    fn render_recent_files(state: &ProgressState, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(40), // Current & Upcoming
                    Constraint::Percentage(30), // Completed
                    Constraint::Percentage(30), // Stats
                ]
                .as_ref(),
            )
            .split(area);

        // Left section: Current and upcoming files
        let mut current_items: Vec<ListItem> = Vec::new();

        // Add current file if it exists
        if !state.current_file.is_empty() {
            let progress_percentage = if state.current_file_size > 0 {
                (state.current_file_progress as f64 / state.current_file_size as f64 * 100.0) as u64
            } else {
                0
            };

            current_items.push(ListItem::new(Line::from(vec![
                Span::styled("⟳ ", Style::default().fg(Color::Yellow)),
                Span::raw(&state.current_file),
                Span::styled(
                    format!(
                        " ({}/{})",
                        format_size(state.current_file_progress),
                        format_size(state.current_file_size)
                    ),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!(" {}%", progress_percentage),
                    Style::default().fg(Color::Yellow),
                ),
            ])));

            // Calculate remaining space for upcoming files
            let remaining_height = area.height as usize - 3; // Account for borders and title
            let files_that_fit = remaining_height.saturating_sub(current_items.len());

            // Show upcoming files from the queue
            let start_idx = state.current_file_index as usize;
            for (filename, size) in state
                .file_queue
                .iter()
                .skip(start_idx + 1)
                .take(files_that_fit)
            {
                current_items.push(ListItem::new(Line::from(vec![
                    Span::styled("• ", Style::default().fg(Color::DarkGray)),
                    Span::raw(filename),
                    Span::styled(
                        format!(" ({})", format_size(*size)),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])));
            }
        }

        // If no files at all, show a message
        if current_items.is_empty() {
            current_items.push(ListItem::new(Line::from(vec![Span::styled(
                "No files in queue...",
                Style::default().fg(Color::DarkGray),
            )])));
        }

        let current_list = List::new(current_items)
            .block(
                Block::default()
                    .title(Span::styled(
                        "Current & Upcoming Files",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        // Middle section: Completed files
        let mut completed_items: Vec<ListItem> = Vec::new();
        let completed_height = area.height as usize - 3;

        for file in state.recent_files.iter().rev().take(completed_height) {
            completed_items.push(ListItem::new(Line::from(vec![
                Span::styled("✓ ", Style::default().fg(Color::Green)),
                Span::raw(file),
                Span::styled(" (100%)", Style::default().fg(Color::Green)),
            ])));
        }

        if completed_items.is_empty() {
            completed_items.push(ListItem::new(Line::from(vec![Span::styled(
                "No files completed yet...",
                Style::default().fg(Color::DarkGray),
            )])));
        }

        let completed_list = List::new(completed_items)
            .block(
                Block::default()
                    .title(Span::styled(
                        "Completed Files",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        // Right section: Stats
        let elapsed = state.start_time.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();

        let files_per_second = if elapsed_secs > 0.0 {
            state.current_file_index as f64 / elapsed_secs
        } else {
            0.0
        };

        let avg_file_size = if state.current_file_index > 0 {
            state.total_bytes as f64 / state.current_file_index as f64
        } else {
            0.0
        };

        let peak_speed = state.bytes_per_second.max(0.0);
        let current_speed = state.bytes_per_second;

        let stats = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Current Speed: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{}/s", format_size(current_speed as u64))),
            ]),
            Line::from(vec![
                Span::styled("Peak Speed: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{}/s", format_size(peak_speed as u64))),
            ]),
            Line::from(vec![
                Span::styled("Elapsed: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!(
                    "{}:{:02}:{:02}",
                    elapsed.as_secs() / 3600,
                    (elapsed.as_secs() % 3600) / 60,
                    elapsed.as_secs() % 60
                )),
            ]),
            Line::from(vec![
                Span::styled("ETA: ", Style::default().fg(Color::Yellow)),
                Span::raw(match state.estimated_time {
                    Some(eta) => format!(
                        "{}:{:02}:{:02}",
                        (eta as u64) / 3600,
                        ((eta as u64) % 3600) / 60,
                        (eta as u64) % 60
                    ),
                    None => "calculating...".to_string(),
                }),
            ]),
            Line::from(vec![
                Span::styled("Files/sec: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{:.2}", files_per_second)),
            ]),
            Line::from(vec![
                Span::styled("Avg File Size: ", Style::default().fg(Color::Yellow)),
                Span::raw(format_size(avg_file_size as u64)),
            ]),
        ])
        .block(
            Block::default()
                .title(Span::styled(
                    "Statistics",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL),
        );

        // Render all sections
        f.render_widget(current_list, chunks[0]);
        f.render_widget(completed_list, chunks[1]);
        f.render_widget(stats, chunks[2]);
    }
}

impl Drop for ProgressUI {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture,
        )
        .unwrap();
    }
}

pub enum ProgressUpdate {
    File {
        name: String,
        size: u64,
        progress: u64,
        index: u64,
        bytes_per_second: f64,
        total_bytes: u64,
        estimated_time: Option<f64>,
    },
    Stop,
    Complete,
}

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
