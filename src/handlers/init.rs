use colored::*;
use std::{process, collections::HashSet, path::PathBuf};
use crate::{
    models::SaveState,
    organizer::get_all_files,
    ui::{get_output_location, get_output_choice},
};

pub struct InitResult {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub files: Vec<crate::models::CustomFile>,
    pub resume_path: Option<PathBuf>,
    pub save_state: Option<SaveState>,
}

pub fn initialize_app(operation_mode: Option<PathBuf>) -> InitResult {
    match operation_mode {
        Some(save_path) => match SaveState::load(&save_path) {
            Ok(save_state) => {
                println!("{}", "📝 Resuming from save file...".bright_green());
                println!(
                    "{} {}",
                    "Input folder:".green(),
                    save_state.input_path.display()
                );
                println!(
                    "{} {}",
                    "Output folder:".green(),
                    save_state.output_path.display()
                );

                let processed_paths: HashSet<_> = save_state
                    .processed_files
                    .iter()
                    .map(|f| f.path.clone())
                    .collect();

                let all_files = get_all_files(&save_state.input_path);
                let remaining_files: Vec<_> = all_files
                    .into_iter()
                    .filter(|f| !processed_paths.contains(&f.path))
                    .collect();

                InitResult {
                    input_path: save_state.input_path.clone(),
                    output_path: save_state.output_path.clone(),
                    files: remaining_files,
                    resume_path: Some(save_path),
                    save_state: Some(save_state),
                }
            }
            Err(e) => {
                eprintln!("{} {}", "Failed to load save file:".red(), e);
                process::exit(1);
            }
        },
        None => {
            let input_path = get_output_location()
                .map(|paths| paths.input_path)
                .expect("Failed to get input location");

            println!(
                "{} {}",
                "Selected input folder:".green(),
                input_path.display()
            );

            let output_path = get_output_choice(&input_path);
            println!(
                "{} {}",
                "Selected output folder:".green(),
                output_path.display()
            );

            println!("\n{}", "🔍 Scanning files...".bright_cyan());
            let files = get_all_files(&input_path);

            InitResult {
                input_path,
                output_path,
                files,
                resume_path: None,
                save_state: None,
            }
        }
    }
}