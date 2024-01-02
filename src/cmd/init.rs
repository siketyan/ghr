use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use console::style;
use dialoguer::Confirm;
use git2::Repository;
use tracing::info;

use crate::config::Config;
use crate::path::Path;
use crate::root::Root;
use crate::url::Url;

#[derive(Debug, Parser)]
pub struct Cmd {
    /// URL or pattern of the repository to clone.
    repo: String,

    /// Change directory after cloned a repository (Shell extension required).
    #[clap(long)]
    cd: bool,

    /// Opens the directory after cloned a repository.
    #[clap(long)]
    open: Option<Option<String>>,
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
        let path = Path::resolve(&root, &url);
        let profile = config
            .rules
            .resolve(&url)
            .and_then(|r| config.profiles.resolve(&r.profile));

        let path = PathBuf::from(&path);
        if path.exists()
            && !Confirm::new()
                .with_prompt(format!(
                    "{} The directory already exists. Are you sure want to re-initialise?",
                    style("CHECK").dim(),
                ))
                .interact()?
        {
            return Ok(());
        }

        let repo = Repository::init(&path)?;

        info!(
            "Initialised a repository successfully in: {}",
            repo.workdir().unwrap().to_string_lossy(),
        );

        if let Some((name, p)) = profile {
            p.apply(&repo)?;

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
