use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use console::style;
use git2::Repository;
use indicatif::{ProgressBar, ProgressStyle};
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
    pub async fn run(self) -> Result<()> {
        let root = Root::find()?;
        let config = Config::load_from(&root)?;

        let (tx, rx) = channel();
        let progress = tokio::spawn(async move {
            let spinner = ProgressStyle::with_template("{prefix} {spinner} {wide_msg}")
                .unwrap()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

            let p = ProgressBar::new(u64::MAX)
                .with_style(spinner)
                .with_prefix(format!(" {}", style("WAIT").dim()))
                .with_message("Cloning the repository...");

            while rx.recv_timeout(Duration::from_millis(100)).is_err() {
                p.tick();
            }

            p.finish_and_clear();
        });

        let url = Url::from_str(&self.repo)?;
        let path = Path::resolve(&root, &url);
        let profile = config
            .rules
            .resolve(&url)
            .and_then(|r| config.profiles.resolve(&r.profile));

        let repo = Repository::clone(&url.to_string(), PathBuf::from(&path))?;
        if let Some((name, p)) = profile {
            p.apply(&mut repo.config()?)?;

            info!("Attached profile [{}] successfully.", style(name).bold());
        }

        tx.send(())?;
        progress.await?;

        info!(
            "Cloned a repository successfully to: {:?}",
            repo.workdir().unwrap(),
        );

        Ok(())
    }
}
