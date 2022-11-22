mod config;
mod strategy;

pub use config::Config;

use std::path::Path;

use anyhow::Result;

pub trait CloneRepository {
    fn clone_repository<U, P>(&self, url: U, path: P) -> Result<()>
    where
        U: ToString,
        P: AsRef<Path>;
}
