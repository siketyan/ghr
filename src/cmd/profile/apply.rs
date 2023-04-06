use anyhow::{anyhow, Result};
use clap::Parser;
use console::style;
use git2::Repository;
use tracing::info;

use crate::config::Config;

#[derive(Debug, Parser)]
pub struct Cmd {
    // Name of the profile to apply.
    name: String,
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        let config = Config::load()?;
        let profile = config
            .profiles
            .get(&self.name)
            .ok_or_else(|| anyhow!("Unknown profile: {}", &self.name))?;

        let repo = Repository::open_from_env()?;

        profile.apply(&mut repo.config()?)?;
        info!("Attached profile [{}] successfully.", style(self.name).bold());

        Ok(())
    }
}
