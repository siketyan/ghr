mod cli;

pub use cli::Cli;

use std::path::Path;

use serde::Deserialize;

use crate::git::{CloneOptions, CloneRepository, Progress};

#[derive(Debug, Default, Deserialize)]
pub enum Strategy {
    #[default]
    Cli,
}

impl CloneRepository for Strategy {
    fn clone_repository(
        &self,
        url: impl ToString,
        path: impl AsRef<Path>,
        progress: impl Progress,
        options: &CloneOptions,
    ) -> anyhow::Result<()> {
        match self {
            Self::Cli => Cli.clone_repository(url, path, progress, options),
        }
    }
}
