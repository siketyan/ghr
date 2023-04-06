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
                    .configs.0.get("user.name")
                    .map(|name| name.as_str())
                    .unwrap_or(INHERIT),
                style(&format!(
                    "<{}>",
                    profile
                        .configs.0.get("user.email")
                        .map(|email| email.as_str())
                        .unwrap_or(INHERIT),
                ))
                .dim(),
            );
        });

        Ok(())
    }
}
