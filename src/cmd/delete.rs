use std::future::ready;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::Parser;
use console::style;
use dialoguer::Confirm;
use tracing::info;

use crate::config::Config;
use crate::console::Spinner;
use crate::path::Path;
use crate::root::Root;
use crate::url::{PartialUrl, Url};

#[derive(Debug, Parser)]
pub struct Cmd {
    /// URL or pattern of the repository to delete.
    repo: Vec<String>,
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

        for repo in self.repo.iter() {
            let url = PartialUrl::from_str(repo, &config.patterns)?;
            let path = config
                .search_path
                .owner
                .iter()
                .map(|default_owner| Url::from_partial(&url, Some(default_owner)).unwrap())
                .map(|u| PathBuf::from(Path::resolve(&root, &u)))
                .find(|p| p.exists())
                .ok_or_else(|| anyhow!("The repository does not exist on the filesystem."))?;

            Spinner::new("Deleting the repository...")
                .spin_while(|| ready(std::fs::remove_dir_all(&path).map_err(anyhow::Error::from)))
                .await?;

            info!(
                "Deleted the repository successfully: {}",
                path.to_string_lossy(),
            );
        }

        Ok(())
    }
}
