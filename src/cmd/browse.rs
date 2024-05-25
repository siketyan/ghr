use anyhow::{anyhow, bail, Result};
use clap::Parser;

use crate::config::Config;
use crate::git::repository_exists;
use crate::root::Root;
use crate::url::{PartialUrl, Url};

#[cfg(windows)]
fn open_url(url: &url::Url) -> Result<()> {
    use std::ffi::CString;

    use windows::core::{PCSTR, s};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Shell::ShellExecuteA;
    use windows::Win32::UI::WindowsAndMessaging::SHOW_WINDOW_CMD;

    // https://github.com/pkg/browser/issues/16
    // https://github.com/cli/browser/commit/28dca726a60e5e7cdf0326436aa1cb4d476c3305
    // https://web.archive.org/web/20150421233040/https://support.microsoft.com/en-us/kb/224816
    unsafe {
        ShellExecuteA(
            HWND::default(),
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
    repo: String,
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        let root = Root::find()?;
        let config = Config::load_from(&root)?;

        let url = PartialUrl::from_str(&self.repo, &config.patterns)?;
        let url = config
            .search_path
            .owner
            .iter()
            .map(|default_owner| Url::from_partial(&url, Some(default_owner)).unwrap())
            .find_map(|u| match repository_exists(&u) {
                Ok(true) => Some(Ok(u)),
                Ok(false) => None,
                Err(e) => Some(Err(e)),
            });

        let url = match url {
            Some(Ok(u)) => u,
            Some(Err(e)) => return Err(e),
            _ => bail!("Could not find the repository on the remote. Check your search_path config and the repository exists.")
        };

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
