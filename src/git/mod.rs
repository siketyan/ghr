mod config;
mod strategy;

pub use config::Config;

use std::path::Path;

use anyhow::Result;

#[derive(Debug, Default)]
pub struct CloneOptions {
    pub recursive: bool,
}

pub trait CloneRepository {
    fn clone_repository<U, P>(&self, url: U, path: P, options: &CloneOptions) -> Result<()>
    where
        U: ToString,
        P: AsRef<Path>;
}
