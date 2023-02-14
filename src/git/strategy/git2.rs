use std::path::Path;

use git2::Repository;
use tracing::{debug, warn};

use crate::git::{CloneOptions, CloneRepository};

pub struct Git2;

impl CloneRepository for Git2 {
    fn clone_repository<U, P>(&self, url: U, path: P, options: &CloneOptions) -> anyhow::Result<()>
    where
        U: ToString,
        P: AsRef<Path>,
    {
        debug!("Cloning the repository using Git2 strategy");
        warn!("Git2 strategy is deprecated and will be removed in v0.4.0. Switch to CLI strategy.");

        let _ = match options.recursive {
            true => Repository::clone_recurse(&url.to_string(), path)?,
            _ => Repository::clone(&url.to_string(), path)?,
        };

        Ok(())
    }
}
