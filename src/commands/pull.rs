use anyhow::{Context, Result};
use clap::Args;
use std::fs;

use crate::config::{config_file, validate_config};
use crate::remote::{RemoteSpec, clone_remote_repo, load_published_config};
use crate::utils::io::prompt_user_default_no;

#[derive(Args, Debug)]
pub struct PullArgs {
    /// Remote reference in the form <id/name>[:tag].
    ///
    /// When the tag is omitted, Vex resolves the latest published version.
    pub remote_ref: String,

    /// Force overwrite when the local configuration already exists.
    #[arg(short = 'f', long = "force")]
    pub force: bool,
}

pub fn pull_command(force: bool, remote_ref: String) -> Result<()> {
    let spec = RemoteSpec::parse(&remote_ref)?;
    let (_temp_dir, worktree) = clone_remote_repo()?;
    let published = load_published_config(&worktree, &spec)?;

    validate_config(&published.config)?;

    let config_path = config_file(&spec.name)?;
    if config_path.exists() && !force {
        println!(
            "Local configuration '{}' already exists, overwrite? [y/N]",
            spec.name
        );
        if !prompt_user_default_no()? {
            println!("Pull cancelled");
            return Ok(());
        }
    }

    let config_json = serde_json::to_string_pretty(&published.config)
        .context("Failed to serialize pulled configuration")?;
    fs::write(&config_path, config_json).context("Failed to save pulled configuration")?;

    println!(
        "Pulled configuration '{} / {}:{}' into {:?}",
        spec.id, spec.name, published.tag, config_path
    );

    Ok(())
}
