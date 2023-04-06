use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

use anyhow::anyhow;
use tracing::debug;

use crate::git::{CloneOptions, CloneRepository, CloneStatus};

pub struct Cli;

impl CloneRepository for Cli {
    fn clone_repository<U, P, S>(
        &self,
        url: U,
        path: P,
        mut status: S,
        options: &CloneOptions,
    ) -> anyhow::Result<()>
    where
        U: ToString,
        P: AsRef<Path>,
        S: CloneStatus,
    {
        debug!("Cloning the repository using CLI strategy");

        let url = url.to_string();
        let mut args = vec!["clone", &url, path.as_ref().to_str().unwrap()];
        if options.recursive {
            args.push("--recursive");
        }

        let mut child = Command::new("git").args(args).spawn()?;
        let mut stdout = BufReader::new(child.stdout.as_mut().unwrap());
        let mut buffer = String::new();

        while stdout.read_line(&mut buffer).ok().is_some() {
            status.set_clone_status(&buffer);
        }

        let output = child.wait_with_output()?;
        match output.status.success() {
            true => Ok(()),
            _ => Err(anyhow!(
                "Error occurred while cloning the repository: {}",
                String::from_utf8_lossy(output.stderr.as_slice()),
            )),
        }
    }
}
