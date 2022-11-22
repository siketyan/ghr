use std::path::Path;

use git2::Repository;
use tracing::debug;

use crate::git::CloneRepository;

pub struct Git2;

impl CloneRepository for Git2 {
    fn clone_repository<U, P>(&self, url: U, path: P) -> anyhow::Result<()>
    where
        U: ToString,
        P: AsRef<Path>,
    {
        debug!("Cloning the repository using Git2 strategy");

        let _ = Repository::clone(&url.to_string(), path)?;

        Ok(())
    }
}
