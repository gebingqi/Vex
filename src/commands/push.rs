use anyhow::{Context, Result};
use clap::Args;
use std::fs;

use crate::config::{QemuConfig, config_file, validate_config};
use crate::remote::{PublishOutcome, RemoteSpec, publish_config};

#[derive(Args, Debug)]
pub struct PushArgs {
    /// Remote reference in the form <id/name>[:tag].
    ///
    /// When the tag is omitted, the publication updates the latest alias only.
    pub remote_ref: String,

    /// Local configuration name to publish.
    pub local_name: String,

    /// Force overwrite when the remote tag already exists.
    #[arg(short = 'f', long = "force")]
    pub force: bool,
}

pub fn push_command(force: bool, remote_ref: String, local_name: String) -> Result<()> {
    let spec = RemoteSpec::parse(&remote_ref)?;
    let config_path = config_file(&local_name)?;
    if !config_path.exists() {
        anyhow::bail!(
            "Configuration '{}' does not exist. Create it first with 'vex save'",
            local_name
        );
    }

    let config_json = fs::read_to_string(&config_path).context("Failed to read config file")?;
    let config: QemuConfig =
        serde_json::from_str(&config_json).context("Failed to deserialize configuration")?;

    validate_config(&config)?;

    match publish_config(&spec, &config, force)? {
        PublishOutcome::Cancelled => {}
        PublishOutcome::NoChanges => {
            println!(
                "Remote configuration '{} / {}:{}' is already up to date",
                spec.id,
                spec.name,
                spec.resolved_tag()
            );
        }
        PublishOutcome::Pushed => {
            println!(
                "Pushed local configuration '{}' to '{} / {}:{}'",
                local_name,
                spec.id,
                spec.name,
                spec.resolved_tag()
            );
        }
    }

    Ok(())
}
