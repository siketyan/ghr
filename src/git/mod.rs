mod config;
mod strategy;

pub use config::Config;

use std::path::Path;

use anyhow::Result;

pub trait CloneStatus {
    fn set_clone_status(&mut self, message: &str);
}

#[derive(Debug, Default)]
pub struct CloneOptions {
    pub recursive: bool,
}

pub trait CloneRepository {
    fn clone_repository<U, P, S>(
        &self,
        url: U,
        path: P,
        status: S,
        options: &CloneOptions,
    ) -> Result<()>
    where
        U: ToString,
        P: AsRef<Path>,
        S: CloneStatus;
}
