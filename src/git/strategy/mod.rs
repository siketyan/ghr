mod cli;

pub use cli::Cli;

use std::path::Path;

use serde::Deserialize;

use crate::git::{CloneOptions, CloneRepository};

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
