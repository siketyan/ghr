use std::path::Path;
use std::process::Command;

use anyhow::anyhow;
use tracing::debug;

use crate::git::{CloneOptions, CloneRepository};

pub struct Cli;

impl CloneRepository for Cli {
    fn clone_repository<U, P>(&self, url: U, path: P, options: &CloneOptions) -> anyhow::Result<()>
    where
        U: ToString,
        P: AsRef<Path>,
    {
        debug!("Cloning the repository using CLI strategy");

        let url = url.to_string();
        let mut args = vec!["clone", &url, path.as_ref().to_str().unwrap()];
        if options.recursive {
            args.push("--recursive");
        }

        let output = Command::new("git").args(args).output()?;
        match output.status.success() {
            true => Ok(()),
            _ => Err(anyhow!(
                "Error occurred while cloning the repository: {}",
                String::from_utf8_lossy(output.stderr.as_slice()).trim(),
            )),
        }
    }
}
