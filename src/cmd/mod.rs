mod clone;
mod path;
mod profile;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Action {
    /// Clones a Git repository to local.
    Clone(clone::Cmd),
    /// Prints the path to root, owner, or a repository.
    Path(path::Cmd),
    /// Manages profiles to use in repositories.
    Profile(profile::Cmd),
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    action: Action,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        use Action::*;
        match self.action {
            Clone(cmd) => cmd.run().await,
            Path(cmd) => cmd.run(),
            Profile(cmd) => cmd.run(),
        }
    }
}
