use std::path::Path;
use std::process::Command;

use anyhow::anyhow;
use tracing::debug;

use crate::git::{CheckoutBranch, CloneOptions, CloneRepository, Fetch};

pub struct Cli;

impl CloneRepository for Cli {
    fn clone_repository<U, P>(&self, url: U, path: P, options: &CloneOptions) -> anyhow::Result<()>
    where
        U: ToString,
        P: AsRef<Path>,
    {
        debug!("Cloning the repository using CLI strategy");

        let mut args = vec![
            "clone".to_string(),
            url.to_string(),
            path.as_ref().to_string_lossy().to_string(),
        ];

        if let Some(recursive) = options.recursive.as_ref() {
            args.push(match recursive.as_deref() {
                Some(path) => format!("--recurse-submodules={path}"),
                _ => "--recurse-submodules".to_string(),
            });
        }
        if options.single_branch {
            args.push("--single-branch".to_string());
        }
        if let Some(origin) = options.origin.as_deref() {
            args.push(format!("--origin={origin}"));
        }
        if let Some(branch) = options.branch.as_deref() {
            args.push(format!("--branch={branch}"));
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

impl Fetch for Cli {
    fn fetch<P>(&self, path: P, remote: impl Into<String>) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        let output = Command::new("git")
            .current_dir(path)
            .args(["fetch".to_string(), remote.into()])
            .output()?;

        match output.status.success() {
            true => Ok(()),
            _ => Err(anyhow!(
                "Error occurred while fetching the remote: {}",
                String::from_utf8_lossy(output.stderr.as_slice()).trim(),
            )),
        }
    }
}

impl CheckoutBranch for Cli {
    fn checkout_branch<P>(
        &self,
        path: P,
        branch: impl Into<String>,
        track: impl Into<Option<String>>,
    ) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        let mut args = Vec::from(["checkout".to_string(), "-b".to_string(), branch.into()]);
        if let Some(t) = track.into() {
            args.push("--track".to_string());
            args.push(t);
        }

        let output = Command::new("git").current_dir(path).args(args).output()?;
        match output.status.success() {
            true => Ok(()),
            _ => Err(anyhow!(
                "Error occurred while fetching the remote: {}",
                String::from_utf8_lossy(output.stderr.as_slice()).trim(),
            )),
        }
    }
}
