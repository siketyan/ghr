use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use console::style;
use git2::Repository;
use tracing::info;

use crate::config::Config;
use crate::console::Spinner;
use crate::git::{CloneOptions, CloneRepository};
use crate::path::Path;
use crate::root::Root;
use crate::url::Url;

#[derive(Debug, Parser)]
pub struct Cmd {
    /// URL or pattern of the repository to clone.
    repo: String,

    /// Clones their submodules recursively.
    #[clap(short, long)]
    recursive: bool,

    /// Change directory after cloned a repository (Shell extension required).
    #[clap(long)]
    cd: bool,

    /// Opens the directory after cloned a repository.
    #[clap(long)]
    open: Option<String>,
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        let root = Root::find()?;
        let config = Config::load_from(&root)?;

        let url = Url::from_str(&self.repo, config.defaults.owner.as_deref())?;
        let path = PathBuf::from(Path::resolve(&root, &url));
        let profile = config
            .rules
            .resolve(&url)
            .and_then(|r| config.profiles.resolve(&r.profile));

        let repo = Spinner::new("Cloning the repository...")
            .spin_while(|| {
                let path = path.clone();
                async move {
                    config.git.strategy.clone.clone_repository(
                        url,
                        &path,
                        &CloneOptions {
                            recursive: self.recursive,
                        },
                    )?;

                    Ok::<_, anyhow::Error>(Repository::open(&path)?)
                }
            })
            .await?;

        info!(
            "Cloned a repository successfully to: {}",
            repo.workdir().unwrap().to_string_lossy(),
        );

        if let Some((name, p)) = profile {
            p.apply(&mut repo.config()?)?;

            info!("Attached profile [{}] successfully.", style(name).bold());
        }

        if let Some(app) = self.open {
            config.applications.open_or_intermediate(&app, &path)?;

            info!(
                "Opened the repository in [{}] successfully.",
                style(&app).bold(),
            );
        }

        Ok(())
    }
}
