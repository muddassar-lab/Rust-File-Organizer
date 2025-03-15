use colored::*;
use dialoguer::{Select, theme::ColorfulTheme};
use std::path::PathBuf;
use crate::{
    OrganizeError,
    models::SaveState,
    ui::cleanup,
    cli::handle_error,
    save::{save_progress, handle_save_cleanup},
};

pub fn handle_organization_result(
    result: Result<Option<SaveState>, OrganizeError>,
    resume_path: Option<PathBuf>,
    output_path: PathBuf,
) {
    match result {
        Ok(Some(mut save_state)) => {
            println!("\n{}", "🛑 Process interrupted!".yellow());

            match resume_path {
                Some(path) => {
                    if let Ok(mut existing_save) = SaveState::load(&path) {
                        existing_save
                            .processed_files
                            .extend(save_state.processed_files);
                        save_state = existing_save;
                    }

                    if let Err(e) =
                        std::fs::write(&path, serde_json::to_string_pretty(&save_state).unwrap())
                    {
                        handle_error(
                            OrganizeError::FileCopyFailed(format!(
                                "Failed to update save file: {}",
                                e
                            )),
                            Some(&output_path),
                        );
                    }
                    println!(
                        "\n{}",
                        "📝 Progress automatically saved to existing file.".bright_yellow()
                    );
                    println!("\n{}", "👋 Goodbye!".bright_blue());
                }
                None => {
                    let options = vec!["Save progress and exit", "Just exit"];
                    let selection = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("What would you like to do?")
                        .items(&options)
                        .default(0)
                        .interact()
                        .unwrap_or(1);

                    match selection {
                        0 => {
                            if let Err(e) = save_progress(save_state) {
                                handle_error(e, Some(&output_path));
                            }
                            println!("\n{}", "👋 Goodbye!".bright_blue());
                        }
                        _ => {
                            cleanup(&output_path);
                            println!("\n{}", "👋 Goodbye!".bright_blue());
                        }
                    }
                }
            }
        }
        Ok(None) => {
            println!(
                "\n{}",
                "✨ Organization completed successfully!"
                    .bright_green()
                    .bold()
            );
            println!(
                "{} {}",
                "📍 Files organized at:".bright_cyan(),
                output_path.display()
            );

            handle_save_cleanup(resume_path);
        }
        Err(e) => handle_error(e, Some(&output_path)),
    }
}