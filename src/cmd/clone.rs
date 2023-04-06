use std::path::PathBuf;
use std::sync::mpsc::Sender;

use anyhow::Result;
use clap::Parser;
use console::style;
use git2::Repository;
use itertools::Itertools;
use tracing::info;

use crate::config::Config;
use crate::console::{Message, Spinner};
use crate::git::{CloneOptions, CloneRepository, CloneStatus};
use crate::path::Path;
use crate::root::Root;
use crate::url::Url;

struct ProgressBarStatus {
    tx: Sender<Message>,
}

impl CloneStatus for &ProgressBarStatus {
    fn set_clone_status(&mut self, message: &str) {
        self.tx
            .send(Message::UpdateText(message.to_string()))
            .unwrap();
    }
}

#[derive(Debug, Parser)]
pub struct Cmd {
    /// URL or pattern of the repository to clone.
    repo: Vec<String>,

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

        let repo: Vec<CloneResult> = Spinner::new("Cloning the repository...")
            .spin_while(|tx| async move {
                let status = ProgressBarStatus { tx };
                self.repo
                    .iter()
                    .map(|repo| self.clone(&root, &config, repo, &status))
                    .try_collect()
            })
            .await?;

        repo.iter().for_each(
            |CloneResult {
                 path,
                 profile,
                 open,
             }| {
                info!(
                    "Cloned a repository successfully to: {}",
                    path.to_string_lossy(),
                );

                if let Some(name) = profile {
                    info!(
                        "\t-> Attached profile [{}] successfully.",
                        style(name).bold()
                    );
                }

                if let Some(app) = open {
                    info!(
                        "\t-> Opened the repository in [{}] successfully.",
                        style(&app).bold(),
                    );
                }
            },
        );

        Ok(())
    }

    fn clone(
        &self,
        root: &Root,
        config: &Config,
        repo: &str,
        status: &ProgressBarStatus,
    ) -> Result<CloneResult> {
        let url = Url::from_str(repo, &config.patterns, config.defaults.owner.as_deref())?;
        let path = PathBuf::from(Path::resolve(root, &url));
        let profile = config
            .rules
            .resolve(&url)
            .and_then(|r| config.profiles.resolve(&r.profile));

        info!("Cloning from '{}'", url.to_string());

        config.git.strategy.clone.clone_repository(
            url,
            &path,
            status,
            &CloneOptions {
                recursive: self.recursive,
            },
        )?;

        let repo = Repository::open(&path)?;
        let profile = if let Some((name, p)) = profile {
            p.apply(&mut repo.config()?)?;
            Some(name.to_string())
        } else {
            None
        };

        let open = if let Some(app) = &self.open {
            config.applications.open_or_intermediate(app, &path)?;
            Some(app.to_string())
        } else {
            None
        };

        Ok(CloneResult {
            path: repo.workdir().unwrap().to_path_buf(),
            profile,
            open,
        })
    }
}

struct CloneResult {
    path: PathBuf,
    profile: Option<String>,
    open: Option<String>,
}
