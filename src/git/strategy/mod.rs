mod cli;

pub use cli::Cli;

use std::path::Path;

use serde::Deserialize;

use crate::git::{CheckoutBranch, CloneOptions, CloneRepository, Fetch};

#[derive(Debug, Default, Deserialize)]
pub enum Strategy {
    #[default]
    Cli,
}

impl CloneRepository for Strategy {
    fn clone_repository<U, P>(&self, url: U, path: P, options: &CloneOptions) -> anyhow::Result<()>
    where
        U: ToString,
        P: AsRef<Path>,
    {
        match self {
            Self::Cli => Cli.clone_repository(url, path, options),
        }
    }
}

impl Fetch for Strategy {
    fn fetch<P>(&self, path: P, remote: impl Into<String>) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        match self {
            Self::Cli => Cli.fetch(path, remote),
        }
    }
}

impl CheckoutBranch for Strategy {
    fn checkout_branch<P>(
        &self,
        path: P,
        branch: impl Into<String>,
        track: impl Into<Option<String>>,
    ) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        match self {
            Self::Cli => Cli.checkout_branch(path, branch, track),
        }
    }
}
