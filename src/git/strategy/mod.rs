mod cli;

pub use cli::Cli;

use std::path::Path;

use serde::Deserialize;

use crate::git::{CloneOptions, CloneRepository, CloneStatus};

#[derive(Debug, Default, Deserialize)]
pub enum Strategy {
    #[default]
    Cli,
}

impl CloneRepository for Strategy {
    fn clone_repository<U, P, S>(
        &self,
        url: U,
        path: P,
        status: S,
        options: &CloneOptions,
    ) -> anyhow::Result<()>
    where
        U: ToString,
        P: AsRef<Path>,
        S: CloneStatus,
    {
        match self {
            Self::Cli => Cli.clone_repository(url, path, status, options),
        }
    }
}
