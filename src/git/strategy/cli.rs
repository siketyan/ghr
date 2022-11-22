use std::path::Path;
use std::process::Command;

use tracing::debug;

use crate::git::CloneRepository;

pub struct Cli;

impl CloneRepository for Cli {
    fn clone_repository<U, P>(&self, url: U, path: P) -> anyhow::Result<()>
    where
        U: ToString,
        P: AsRef<Path>,
    {
        debug!("Cloning the repository using CLI strategy");

        let _ = Command::new("git")
            .args(["clone", &url.to_string(), path.as_ref().to_str().unwrap()])
            .output()?;

        Ok(())
    }
}
