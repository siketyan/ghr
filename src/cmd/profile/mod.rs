mod apply;
mod list;
mod show;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Action {
    /// Lists all configured profiles.
    List(list::Cmd),
    /// Shows a profile in TOML format.
    Show(show::Cmd),
    /// Apply a profile.
    Apply(apply::Cmd),
}

#[derive(Debug, Parser)]
pub struct Cmd {
    #[clap(subcommand)]
    action: Action,
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        use Action::*;
        match self.action {
            List(cmd) => cmd.run(),
            Show(cmd) => cmd.run(),
            Apply(cmd) => cmd.run(),
        }
    }
}
