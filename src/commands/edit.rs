use anyhow::{Context, Result};
use clap::Args;
use std::env;
use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::Builder;

use crate::commands::exec::exec_command;
use crate::config::{QemuConfig, config_file};
// 引入 prompt_user_default_no
use crate::utils::io::prompt_user_default_no;

#[derive(Args, Debug)]
/// Edit a saved configuration interactively
///
/// Opens the JSON configuration file in your system's default text editor ($EDITOR).
///
/// Vex uses a secure editing workflow: it modifies a temporary file and strictly
/// validates the JSON syntax before applying any changes to your actual configuration.
/// After a successful edit, it optionally allows you to test-run the VM.
///
/// # Examples
///
/// Edit a configuration:
///   vex edit my-vm
pub struct EditArgs {
    pub name: String,
}

pub fn edit_command(name: String) -> Result<()> {
    let config_path = config_file(&name)?;
    if !config_path.exists() {
        anyhow::bail!("Configuration '{}' does not exist. Cannot edit.", name);
    }

    let original_content =
        fs::read_to_string(&config_path).context("Failed to read config file")?;

    let mut temp_file = Builder::new()
        .prefix("vex-edit-")
        .suffix(".json")
        .tempfile()
        .context("Failed to create temporary file")?;

    temp_file.write_all(original_content.as_bytes())?;

    let temp_path = temp_file.into_temp_path();

    // Determine which editor to use (fallback to vim or notepad)
    let editor = env::var("EDITOR").unwrap_or_else(|_| {
        if cfg!(windows) {
            "notepad".to_string()
        } else {
            "vim".to_string()
        }
    });

    let status = if cfg!(windows) {
        Command::new("cmd")
            .arg("/C")
            .arg(format!("{} \"{}\"", editor, temp_path.display()))
            .status()
            .with_context(|| format!("Failed to open editor: {}", editor))?
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(format!("{} \"{}\"", editor, temp_path.display()))
            .status()
            .with_context(|| format!("Failed to open editor: {}", editor))?
    };

    if !status.success() {
        anyhow::bail!("Editor exited with an error status.");
    }

    // Read the edited content back
    let edited_content = fs::read_to_string(&temp_path)?;

    // If no changes were made, exit early
    if original_content == edited_content {
        println!("No changes made to '{}'.", name);
        return Ok(());
    }

    // Validate the new content is correct JSON
    match serde_json::from_str::<QemuConfig>(&edited_content) {
        Ok(new_config) => {
            // It's valid! Format it nicely and overwrite the original file
            let formatted_json = serde_json::to_string_pretty(&new_config)?;
            fs::write(&config_path, formatted_json).context("Failed to save updated config")?;
            println!("Configuration '{}' updated successfully.", name);

            println!("\nDo you want to test-run this configuration now? [y/N]");
            if prompt_user_default_no()? {
                println!("Starting test-run for '{}'...\n", name);
                // Call the existing exec_command logic
                exec_command(name, false, false)?;
            }
        }
        Err(e) => {
            // Abort if the user messed up the JSON syntax
            anyhow::bail!("Invalid JSON configuration. Edit aborted. Error: {}", e);
        }
    }

    Ok(())
}
