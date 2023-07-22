use std::fs::{create_dir_all, rename};
use std::path::PathBuf;

use anyhow::{anyhow, bail, Result};
use clap::Parser;
use console::style;
use dialoguer::Confirm;
use git2::{Remote, Repository};
use itertools::Itertools;
use tracing::info;

use crate::config::Config;
use crate::path::Path;
use crate::root::Root;
use crate::url::Url;

#[derive(Debug, Parser)]
pub struct Cmd {
    /// Path to the repository to add.
    repo: String,

    /// Forces to add the repository without any prompt.
    #[clap(short, long)]
    force: bool,

    /// Change directory after added the repository. (Shell extension required)
    #[clap(long)]
    cd: bool,

    /// Opens the directory after added the repository.
    #[clap(long)]
    open: Option<Option<String>>,
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        let root = Root::find()?;
        let config = Config::load_from(&root)?;

        let repo = Repository::open(&self.repo)?;
        let remotes: Vec<Remote> = repo
            .remotes()?
            .iter()
            .flatten()
            .map(|r| repo.find_remote(r))
            .try_collect()?;

        let url =
            match remotes.iter().filter_map(|r| r.url()).find_map(|u| {
                Url::from_str(u, &config.patterns, config.defaults.owner.as_deref()).ok()
            }) {
                Some(u) => u,
                _ => bail!("Could not find a supported remote in the repository."),
            };

        let _ = repo; // Closing the repository

        let path = Path::resolve(&root, &url);
        let profile = config
            .rules
            .resolve(&url)
            .and_then(|r| config.profiles.resolve(&r.profile));

        let path = PathBuf::from(&path);

        info!("URL of the repository is: {}", url.to_string());
        info!(
            "This will move entire the repository to: {}",
            path.to_string_lossy(),
        );

        if !self.force
            && !Confirm::new()
                .with_prompt(format!(
                    "{} Are you sure want to continue?",
                    style("CHECK").dim(),
                ))
                .interact()?
        {
            return Ok(());
        }

        let parent_path = path
            .parent()
            .ok_or_else(|| {
                anyhow!(
                    "Failed to determine parent path for the repository's new location: {}",
                    path.to_string_lossy()
                )
            })?
            .to_path_buf();

        create_dir_all(&parent_path)?;

        rename(&self.repo, &path)?;
        info!(
            "Added the repository successfully to: {}",
            path.to_string_lossy(),
        );

        let repo = Repository::open(&path)?;

        if let Some((name, p)) = profile {
            p.apply(&mut repo.config()?)?;

            info!("Attached profile [{}] successfully.", style(name).bold());
        }

        if let Some(app) = self.open {
            config
                .applications
                .open_or_intermediate_or_default(app.as_deref(), &path)?;

            info!(
                "Opened the repository in [{}] successfully.",
                style(app.as_deref().unwrap_or("<default>")).bold(),
            );
        }

        Ok(())
    }
}
