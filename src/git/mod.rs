mod config;
mod strategy;

pub use config::Config;

use std::path::Path;

use anyhow::Result;

#[derive(Debug, Default)]
pub struct CloneOptions {
    pub recursive: Option<Option<String>>,
    pub single_branch: bool,
    pub origin: Option<String>,
    pub branch: Option<String>,
}

pub trait CloneRepository {
    fn clone_repository<U, P>(&self, url: U, path: P, options: &CloneOptions) -> Result<()>
    where
        U: ToString,
        P: AsRef<Path>;
}

pub trait Fetch {
    fn fetch<P>(&self, path: P, remote: impl Into<String>) -> Result<()>
    where
        P: AsRef<Path>;
}

pub trait CheckoutBranch {
    fn checkout_branch<P>(
        &self,
        path: P,
        branch: impl Into<String>,
        track: impl Into<Option<String>>,
    ) -> Result<()>
    where
        P: AsRef<Path>;
}
