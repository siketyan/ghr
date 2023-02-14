use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::config::Config;
use crate::path::Path;
use crate::root::Root;
use crate::url::Url;

#[derive(Debug, Parser)]
pub struct Cmd {
    /// URL or pattern of the repository to open application in.
    repo: String,

    /// Name of the application entry.
    application: String,
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        let root = Root::find()?;
        let config = Config::load_from(&root)?;

        let url = Url::from_str(
            &self.repo,
            &config.patterns,
            config.defaults.owner.as_deref(),
        )?;
        let path = PathBuf::from(Path::resolve(&root, &url));

        config
            .applications
            .open_or_intermediate(&self.application, path)?;

        Ok(())
    }
}
