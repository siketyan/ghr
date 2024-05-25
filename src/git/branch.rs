use anyhow::anyhow;
use std::path::Path;
use std::process::Command;

pub fn checkout_branch<P>(
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
