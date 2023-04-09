use std::process::Command;

use anyhow::Result;
use clap::Parser;

use crate::config::Config;
use crate::root::Root;
use crate::url::{Scheme, Url};

#[derive(Debug, Parser)]
pub struct Cmd {
    /// URL or pattern of the repository to be browsed.
    repo: String,
}

#[cfg(windows)]
const URL_OPEN_CMD: &str = "start.exe";

#[cfg(not(windows))]
const URL_OPEN_CMD: &str = "open";

impl Cmd {
    pub fn run(self) -> Result<()> {
        let root = Root::find()?;
        let config = Config::load_from(&root)?;

        let mut url = Url::from_str(
            &self.repo,
            &config.patterns,
            config.defaults.owner.as_deref(),
        )?;
        url.scheme = Scheme::Https;

        Command::new(URL_OPEN_CMD).args([url.to_string()]).spawn()?;
        Ok(())
    }
}
