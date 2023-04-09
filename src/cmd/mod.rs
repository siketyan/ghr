mod browse;
mod cd;
mod clone;
mod delete;
mod init;
mod list;
mod open;
mod path;
mod profile;
mod shell;
mod version;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Action {
    /// Change directory to one of the managed repositories (Shell extension required).
    Cd(cd::Cmd),
    /// Clones a Git repository to local.
    Clone(clone::Cmd),
    /// Deletes a repository from local.
    Delete(delete::Cmd),
    /// Initialises a Git repository in local.
    Init(init::Cmd),
    /// Lists all managed repositories.
    List(list::Cmd),
    /// Opens a repository in an application.
    Open(open::Cmd),
    /// Browse a repository on web.
    Browse(browse::Cmd),
    /// Prints the path to root, owner, or a repository.
    Path(path::Cmd),
    /// Manages profiles to use in repositories.
    Profile(profile::Cmd),
    /// Writes a shell script to extend ghr features.
    Shell(shell::Cmd),
    /// Prints the version of this application.
    Version(version::Cmd),
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
            Cd(cmd) => cmd.run(),
            Clone(cmd) => cmd.run().await,
            Delete(cmd) => cmd.run().await,
            Init(cmd) => cmd.run(),
            List(cmd) => cmd.run(),
            Open(cmd) => cmd.run(),
            Browse(cmd) => cmd.run(),
            Path(cmd) => cmd.run(),
            Profile(cmd) => cmd.run(),
            Shell(cmd) => cmd.run(),
            Version(cmd) => cmd.run(),
        }
    }
}
