use anyhow::Result;
use clap::Parser;
use console::style;

use crate::config::Config;

const INHERIT: &str = "(inherit)";

#[derive(Debug, Parser)]
pub struct Cmd;

impl Cmd {
    pub fn run(self) -> Result<()> {
        let config = Config::load()?;

        config.profiles.iter().for_each(|(name, profile)| {
            println!(
                "   {} - {}: {} {}",
                style("OK").cyan(),
                style(name).bold(),
                profile
                    .user
                    .as_ref()
                    .and_then(|u| u.name.as_deref())
                    .unwrap_or(INHERIT),
                style(&format!(
                    "<{}>",
                    profile
                        .user
                        .as_ref()
                        .and_then(|u| u.email.as_deref())
                        .unwrap_or(INHERIT),
                ))
                .dim(),
            );
        });

        Ok(())
    }
}
