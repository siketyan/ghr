use anyhow::{anyhow, Result};
use clap::Parser;

use crate::config::Config;

#[derive(Debug, Parser)]
pub struct Cmd {
    // Name of the profile to show.
    name: String,
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        let config = Config::load()?;
        let profile = config
            .profiles
            .get(&self.name)
            .ok_or_else(|| anyhow!("Unknown profile: {}", &self.name))?;

        print!("{}", toml::to_string(profile)?);

        Ok(())
    }
}
