use colored::*;
use dialoguer::{Select, theme::ColorfulTheme};
use std::path::PathBuf;
use crate::ui::get_output_location;

pub fn get_output_choice(input_path: &PathBuf) -> PathBuf {
    println!("\n{}", "📂 Select output location:".bright_cyan());

    let options = vec!["Create folder next to input", "Choose custom location"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&options)
        .default(0)
        .interact()
        .unwrap_or(0);

    if selection == 0 {
        let parent = input_path.parent().unwrap_or(input_path);
        parent.join("Organized_Files")
    } else {
        get_output_location()
            .map(|paths| paths.input_path)
            .unwrap_or_else(|_| {
                eprintln!(
                    "{}",
                    "Failed to get custom location. Using default...".yellow()
                );
                input_path
                    .parent()
                    .unwrap_or(input_path)
                    .join("Organized_Files")
            })
    }
}

pub fn cleanup(output_dir: &PathBuf) {
    println!("{}", "\n🗑️  Cleaning up temporary files...".yellow());
    if let Err(e) = std::fs::remove_dir_all(output_dir) {
        eprintln!(
            "{} {}",
            "Failed to clean up directory:".red(),
            e.to_string().bright_red()
        );
    }
}