use anyhow::{anyhow, Result};
use clap::Parser;

use crate::config::Config;
use crate::root::Root;
use crate::url::Url;

#[cfg(windows)]
fn open_url(url: &url::Url) -> Result<()> {
    // Use start command to open a url.
    // start invokes explorer.exe inside when a path is given,
    // which will open the default web browser if a url is given.
    //
    // explorer.exe seems to not work properly on WSL so we don't invoke it directly
    // c.f. https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/start
    // c.f. https://ss64.com/nt/explorer.html
    // c.f. https://github.com/microsoft/WSL/issues/3832
    std::process::Command::new("cmd.exe")
        .args(["/c", "start", &url.to_string()])
        .spawn()?
        .wait()?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn open_url(url: &url::Url) -> Result<()> {
    std::process::Command::new("open")
        .arg(url.to_string())
        .spawn()?;

    Ok(())
}

#[cfg(all(not(windows), not(target_os = "macos")))]
fn open_url(url: &url::Url) -> Result<()> {
    // c.f. https://github.com/cli/browser/blob/main/browser_linux.go
    let commands = ["xdg-open", "x-www-browser", "www-browser", "wslview"];

    for command in commands {
        match std::process::Command::new(command)
            .arg(url.to_string())
            .spawn()
        {
            Ok(mut child) => {
                child.wait()?;
                return Ok(());
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => continue,
                _ => return Err(e.into()),
            },
        }
    }

    Err(anyhow!("no commands were found to open the url"))
}

#[derive(Debug, Parser)]
pub struct Cmd {
    /// URL or pattern of the repository to be browsed.
    repo: String,
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        let root = Root::find()?;
        let config = Config::load_from(&root)?;

        let url = Url::from_str(
            &self.repo,
            &config.patterns,
            config.defaults.owner.as_deref(),
        )?;

        let platform = config
            .platforms
            .find(&url)
            .ok_or_else(|| anyhow!("Could not find a platform to browse on."))?
            .try_into_platform()?;

        let url = platform.get_browsable_url(&url).await?;

        open_url(&url)?;
        Ok(())
    }
}
