use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use console::style;
use git2::Repository;
use tracing::info;

use crate::config::Config;
use crate::console::create_spinner;
use crate::git::CloneRepository;
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
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        let root = Root::find()?;
        let config = Config::load_from(&root)?;

        let (tx, rx) = channel();
        let progress = tokio::spawn(async move {
            let p = create_spinner("Cloning the repository...");
            while rx.recv_timeout(Duration::from_millis(100)).is_err() {
                p.tick();
            }

            p.finish_and_clear();
        });

        let url = Url::from_str(&self.repo)?;
        let path = PathBuf::from(Path::resolve(&root, &url));
        let profile = config
            .rules
            .resolve(&url)
            .and_then(|r| config.profiles.resolve(&r.profile));

        config.git.strategy.clone.clone_repository(url, &path)?;

        let repo = Repository::open(&path)?;
        if let Some((name, p)) = profile {
            p.apply(&mut repo.config()?)?;

            info!("Attached profile [{}] successfully.", style(name).bold());
        }

        tx.send(())?;
        progress.await?;

        info!(
            "Cloned a repository successfully to: {}",
            repo.workdir().unwrap().to_string_lossy(),
        );

        Ok(())
    }
}
