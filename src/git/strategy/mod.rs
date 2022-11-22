mod cli;
mod git2;

pub use {self::git2::Git2, cli::Cli};

use std::path::Path;

use serde::Deserialize;

use crate::git::CloneRepository;

#[derive(Debug, Default, Deserialize)]
pub enum Strategy {
    #[default]
    Cli,
    Git2,
}

impl CloneRepository for Strategy {
    fn clone_repository<U, P>(&self, url: U, path: P) -> anyhow::Result<()>
    where
        U: ToString,
        P: AsRef<Path>,
    {
        match self {
            Self::Cli => Cli.clone_repository(url, path),
            Self::Git2 => Git2.clone_repository(url, path),
        }
    }
}
