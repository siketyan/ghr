mod dump;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Action {
    /// Dump remotes and the current ref of all repositories.
    Dump(dump::Cmd),
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
            Dump(cmd) => cmd.run(),
        }
    }
}
