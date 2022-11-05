use std::path::PathBuf;
use std::str::FromStr;

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
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        let root = Root::find()?;
        let config = Config::load_from(&root)?;

        let url = Url::from_str(&self.repo)?;
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

        let repo = Repository::init(path)?;
        if let Some((name, p)) = profile {
            p.apply(&mut repo.config()?)?;

            info!("Attached profile [{}] successfully.", style(name).bold());
        }

        info!(
            "Initialised a repository successfully in: {}",
            repo.workdir().unwrap().to_string_lossy(),
        );

        Ok(())
    }
}
