use std::future::ready;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use console::style;
use dialoguer::Confirm;
use tracing::info;

use crate::config::Config;
use crate::console::Spinner;
use crate::path::Path;
use crate::root::Root;
use crate::url::Url;

#[derive(Debug, Parser)]
pub struct Cmd {
    /// URL or pattern of the repository to delete.
    repo: String,
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        let root = Root::find()?;
        let config = Config::load_from(&root)?;

        if !Confirm::new()
            .with_prompt(format!(
                "{} Content of the repository will be deleted permanently. Are you sure want to continue?",
                style("CHECK").dim(),
            ))
            .interact()?
        {
            return Ok(());
        }

        let url = Url::from_str(
            &self.repo,
            &config.patterns,
            config.defaults.owner.as_deref(),
        )?;
        let path = PathBuf::from(Path::resolve(&root, &url));

        Spinner::new("Deleting the repository...")
            .spin_while(|_| ready(std::fs::remove_dir_all(&path).map_err(anyhow::Error::from)))
            .await?;

        info!(
            "Deleted the repository successfully: {}",
            path.to_string_lossy(),
        );

        Ok(())
    }
}
