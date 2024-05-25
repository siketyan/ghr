use anyhow::anyhow;
use std::path::Path;
use std::process::Command;

pub fn fetch<P>(path: P, remote: impl Into<String>) -> anyhow::Result<()>
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
