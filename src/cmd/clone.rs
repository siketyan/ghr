use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use async_hofs::iter::AsyncMapExt;
use clap::Parser;
use console::style;
use git2::Repository;
use tokio::time::sleep;
use tokio_stream::StreamExt;
use tracing::{info, warn};

use crate::config::Config;
use crate::console::{MultiSpinner, Spinner};
use crate::git::{CloneOptions, CloneRepository};
use crate::path::Path;
use crate::root::Root;
use crate::url::Url;

// Constant values taken from implementation of GitHub Cli (gh)
// ref: https://github.com/cli/cli/blob/350011/pkg/cmd/repo/fork/fork.go#L328-L344
const CLONE_RETRY_COUNT: u32 = 3;
const CLONE_RETRY_DURATION: Duration = Duration::from_secs(2);

#[derive(Debug, Parser)]
pub struct Cmd {
    /// URL or pattern of the repository to clone.
    repo: Vec<String>,

    /// Forks the repository in the specified owner (organisation) and clones the forked repo.
    #[clap(long)]
    fork: Option<Option<String>>,

    /// Clones multiple repositories in parallel.
    #[clap(short, long)]
    parallel: bool,

    /// Clones their submodules recursively.
    #[clap(short, long, alias = "recurse-submodules")]
    recursive: Option<Option<String>>,

    /// Clones only the default branch.
    #[clap(long)]
    single_branch: bool,

    /// Uses this name instead of `origin` for the upstream remote.
    #[clap(short, long)]
    origin: Option<String>,

    /// Points the specified branch instead of the default branch after cloned the repository.
    #[clap(short, long)]
    branch: Option<String>,

    /// Change directory after cloning the repository (Shell extension required).
    #[clap(long)]
    cd: bool,

    /// Opens the directory after cloning the repository.
    #[clap(long)]
    open: Option<Option<String>>,
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        let root = Root::find()?;
        let config = Config::load_from(&root)?;

        let urls = self
            .repo
            .iter()
            .async_map(|repo| self.url(&config, repo))
            .collect::<Result<Vec<_>>>()
            .await?;

        match self.parallel {
            true => self.clone_parallel(root, config, urls).await?,
            _ => self.clone_serial(&root, &config, urls).await?,
        }
        .into_iter()
        .for_each(
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

    async fn url(&self, config: &Config, repo: &str) -> Result<Url> {
        let mut url = Url::from_str(repo, &config.patterns, config.defaults.owner.as_deref())?;

        if let Some(owner) = &self.fork {
            info!("Forking from '{}'", url.to_string());

            let platform = config
                .platforms
                .find(&url)
                .ok_or_else(|| anyhow!("Could not find a platform to fork on."))?
                .try_into_platform()?;

            url = Url::from_str(
                &platform.fork(&url, owner.clone()).await?,
                &config.patterns,
                config.defaults.owner.as_deref(),
            )?;
        }

        Ok(url)
    }

    async fn clone_serial(
        self,
        root: &Root,
        config: &Config,
        urls: Vec<Url>,
    ) -> Result<Vec<CloneResult>> {
        Spinner::new("Cloning the repository...")
            .spin_while(|| async move {
                urls.into_iter()
                    .async_map(|url| async {
                        info!("Cloning from '{}'", url.to_string());
                        self.clone(root, config, url).await
                    })
                    .collect::<Result<Vec<_>>>()
                    .await
            })
            .await
    }

    async fn clone_parallel(
        self,
        root: Root,
        config: Config,
        urls: Vec<Url>,
    ) -> Result<Vec<CloneResult>> {
        let this = Arc::new(self);
        let root = Arc::new(root);
        let config = Arc::new(config);

        let mut spinner = MultiSpinner::new();
        for url in urls {
            let this = Arc::clone(&this);
            let root = Arc::clone(&root);
            let config = Arc::clone(&config);

            spinner = spinner.with_spin_while(
                format!("Cloning from {}...", url.to_string()),
                move || async move { this.as_ref().clone(&root, &config, url).await },
            );
        }

        Ok(spinner.collect().await?.into_iter().collect())
    }

    async fn clone(&self, root: &Root, config: &Config, url: Url) -> Result<CloneResult> {
        let path = PathBuf::from(Path::resolve(root, &url));
        let profile = config
            .rules
            .resolve(&url)
            .and_then(|r| config.profiles.resolve(&r.profile));

        let mut retries = 0;
        while let Err(e) = config.git.strategy.clone.clone_repository(
            url.clone(),
            &path,
            &CloneOptions {
                recursive: self.recursive.clone(),
                single_branch: self.single_branch,
                origin: self.origin.clone(),
                branch: self.branch.clone(),
            },
        ) {
            retries += 1;
            if retries > CLONE_RETRY_COUNT {
                return Err(e);
            } else {
                warn!(
                    "Cloning failed. Retrying in {} seconds",
                    CLONE_RETRY_DURATION.as_secs(),
                );
                sleep(CLONE_RETRY_DURATION).await;
            }
        }

        let repo = Repository::open(&path)?;
        let profile = if let Some((name, p)) = profile {
            p.apply(&mut repo.config()?)?;
            Some(name.to_string())
        } else {
            None
        };

        let open = if let Some(app) = &self.open {
            config
                .applications
                .open_or_intermediate_or_default(app.as_deref(), &path)?;

            Some(app.as_deref().unwrap_or("<default>").to_string())
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
