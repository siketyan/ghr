use std::io::stderr;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;

mod add;
mod browse;
mod cd;
mod clone;
mod delete;
mod init;
mod list;
mod open;
mod path;
mod profile;
mod search;
mod shell;
mod sync;
mod version;

#[derive(Debug, Subcommand)]
pub enum Action {
    /// Add an existing repository into the ghr managed directory.
    Add(add::Cmd),
    /// Browse a repository on web.
    Browse(browse::Cmd),
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
    /// Prints the path to root, owner, or a repository.
    Path(path::Cmd),
    /// Manages profiles to use in repositories.
    Profile(profile::Cmd),
    /// Perform a fuzzy search on the repositories list.
    Search(search::Cmd),
    /// Writes a shell script to extend ghr features.
    Shell(shell::Cmd),
    /// Sync repositories between your devices.
    Sync(sync::Cmd),
    /// Prints the version of this application.
    Version(version::Cmd),
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    action: Action,

    /// Operates quietly. Errors will be reported even if this option is enabled.
    #[clap(short, long, global = true)]
    quiet: bool,

    /// Operates verbosely. Traces, debug logs will be reported.
    #[clap(short, long, global = true)]
    verbose: bool,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        tracing_subscriber::fmt()
            .compact()
            .without_time()
            .with_target(false)
            .with_env_filter(
                EnvFilter::builder()
                    .with_default_directive(match (self.quiet, self.verbose) {
                        (true, _) => LevelFilter::ERROR.into(),
                        (_, true) => LevelFilter::TRACE.into(),
                        _ => LevelFilter::INFO.into(),
                    })
                    .from_env_lossy(),
            )
            .with_writer(stderr)
            .init();

        use Action::*;
        match self.action {
            Add(cmd) => cmd.run(),
            Cd(cmd) => cmd.run(),
            Clone(cmd) => cmd.run().await,
            Delete(cmd) => cmd.run().await,
            Init(cmd) => cmd.run(),
            List(cmd) => cmd.run(),
            Open(cmd) => cmd.run(),
            Browse(cmd) => cmd.run().await,
            Path(cmd) => cmd.run(),
            Profile(cmd) => cmd.run(),
            Search(cmd) => cmd.run(),
            Shell(cmd) => cmd.run(),
            Sync(cmd) => cmd.run().await,
            Version(cmd) => cmd.run(),
        }
    }
}
