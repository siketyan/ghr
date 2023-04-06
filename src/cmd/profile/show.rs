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

        let mut profile_keys: Vec<_> = profile.configs.0.keys().collect();

        profile_keys.sort();

        for key in profile_keys {
            println!(r#"{} = "{}""#, key, profile.configs.0[key.as_str()])
        }

        Ok(())
    }
}
