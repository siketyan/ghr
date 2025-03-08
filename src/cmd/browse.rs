use anyhow::{Result, anyhow, bail};
use clap::Parser;
use git2::Repository;

use crate::config::Config;
use crate::root::Root;
use crate::url::Url;

#[cfg(windows)]
fn open_url(url: &url::Url) -> Result<()> {
    use std::ffi::CString;

    use windows::Win32::UI::Shell::ShellExecuteA;
    use windows::Win32::UI::WindowsAndMessaging::SHOW_WINDOW_CMD;
    use windows::core::{PCSTR, s};

    // https://github.com/pkg/browser/issues/16
    // https://github.com/cli/browser/commit/28dca726a60e5e7cdf0326436aa1cb4d476c3305
    // https://web.archive.org/web/20150421233040/https://support.microsoft.com/en-us/kb/224816
    unsafe {
        ShellExecuteA(
            None,
            s!("open"),
            PCSTR::from_raw(CString::new(url.to_string().as_str())?.as_ptr() as *const u8),
            PCSTR::null(),
            PCSTR::null(),
            SHOW_WINDOW_CMD(0),
        );
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn open_url(url: &url::Url) -> Result<()> {
    std::process::Command::new("open")
        .arg(url.to_string())
        .spawn()?
        .wait()?;

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

    let commands = commands.join(", ");
    Err(anyhow!(
        "Command not found: you need one of the following commands to open a url: {commands}"
    ))
}

#[derive(Debug, Parser)]
pub struct Cmd {
    /// URL or pattern of the repository to be browsed.
    /// Defaults to the default remote of the repository at the current directory.
    repo: Option<String>,
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        let root = Root::find()?;
        let config = Config::load_from(&root)?;

        let url = match self.repo.as_deref() {
            Some(path) => path.to_owned(),
            _ => {
                let repo = Repository::open_from_env()?;

                let remotes = repo.remotes()?;
                let remote = match remotes.iter().flatten().next() {
                    Some(r) => r.to_owned(),
                    _ => bail!("The repository has no remote."),
                };

                let remote = repo.find_remote(&remote)?;
                match remote.url() {
                    Some(url) => url.to_string(),
                    _ => bail!("Could not find the remote URL from the repository."),
                }
            }
        };

        let url = Url::from_str(&url, &config.patterns, config.defaults.owner.as_deref())?;

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
