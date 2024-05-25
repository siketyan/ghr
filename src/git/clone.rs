use std::path::Path;
use std::process::Command;

use anyhow::anyhow;
use tracing::debug;

#[derive(Debug, Default)]
pub struct CloneOptions {
    pub recursive: Option<Option<String>>,
    pub single_branch: bool,
    pub origin: Option<String>,
    pub branch: Option<String>,
}

pub fn clone_repository<U, P>(url: U, path: P, options: &CloneOptions) -> anyhow::Result<()>
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
