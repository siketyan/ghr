use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use console::style;
use dialoguer::Confirm;
use tracing::info;

use crate::console::create_spinner;
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

        if !Confirm::new()
            .with_prompt(format!(
                "{} Content of the repository will be deleted permanently. Are you sure want to continue?",
                style("CHECK").dim(),
            ))
            .interact()?
        {
            return Ok(());
        }

        let url = Url::from_str(&self.repo)?;
        let path = PathBuf::from(Path::resolve(&root, &url));

        let (tx, rx) = channel();
        let progress = tokio::spawn(async move {
            let p = create_spinner("Deleting the repository...");
            while rx.recv_timeout(Duration::from_millis(100)).is_err() {
                p.tick();
            }

            p.finish_and_clear();
        });

        std::fs::remove_dir_all(&path)?;
        tx.send(())?;
        progress.await?;

        info!(
            "Deleted the repository successfully: {}",
            path.to_string_lossy(),
        );

        Ok(())
    }
}
