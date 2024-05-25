use std::process::Command;

use anyhow::Result;

use crate::url::Url;

pub fn repository_exists(url: &Url) -> Result<bool> {
    let args = Vec::from(["ls-remote".to_string(), url.to_string()]);
    let output = Command::new("git").args(args).output()?;

    Ok(output.status.success())
}
