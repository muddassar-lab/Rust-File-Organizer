use colored::*;
use dialoguer::{Select, theme::ColorfulTheme};
use std::path::PathBuf;

mod operation;
pub use operation::select_operation_mode;

pub fn print_header() {
    println!("{}", "\n🚀 File Organizer v1.0".bright_blue().bold());
    println!("{}", "===================".bright_blue());
}

pub fn handle_error(error: impl std::fmt::Display, output_path: Option<&std::path::PathBuf>) {
    if let Some(path) = output_path {
        crate::ui::cleanup(path);
    }
    eprintln!("{} {}", "❌ Error:".red(), error.to_string().bright_red());
    std::process::exit(1);
}