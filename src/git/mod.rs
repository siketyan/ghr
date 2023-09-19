mod config;
mod strategy;

pub use config::Config;

use std::path::Path;

use anyhow::Result;

pub trait Progress {
    fn progress(&self, text: &str);
}

#[derive(Debug, Default)]
pub struct CloneOptions {
    pub recursive: Option<Option<String>>,
    pub single_branch: bool,
    pub origin: Option<String>,
    pub branch: Option<String>,
}

pub trait CloneRepository {
    fn clone_repository(
        &self,
        url: impl ToString,
        path: impl AsRef<Path>,
        progress: impl Progress,
        options: &CloneOptions,
    ) -> Result<()>;
}
