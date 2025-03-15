use colored::*;
use dialoguer::{Select, theme::ColorfulTheme};
use std::path::PathBuf;
use crate::models::SaveState;

pub fn select_operation_mode() -> Option<PathBuf> {
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