use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::Parser;

use crate::config::Config;
use crate::path::{PartialPath, Path};
use crate::root::Root;
use crate::url::{Host, Url};

#[derive(Debug, Parser)]
pub struct Cmd {
    /// Remote host of the repository.
    /// Defaults to github.com.
    #[clap(long)]
    host: Option<String>,
    /// Owner name of the repository.
    /// Defaults to the default owner if it is configured.
    #[clap(short, long)]
    owner: Option<String>,
    /// Repository name.
    repo: Option<String>,
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        let root = Root::find()?;
        let config = Config::load_from(&root)?;

        let path = if let Some(repo) = self.repo.as_deref() {
            let url = Url::from_str(
                repo,
                self.owner.as_deref().or(config.defaults.owner.as_deref()),
            )?;

            PathBuf::from(Path::resolve(&root, &url))
        } else {
            PathBuf::from(PartialPath {
                root: &root,
                host: match self.owner.is_some() || self.repo.is_some() {
                    true => self.host.or_else(|| Some(Host::GitHub.to_string())),
                    _ => self.host,
                },
                owner: self.owner,
                repo: None,
            })
        };

        if !path.exists() || !path.is_dir() {
            return Err(anyhow!(
                "The path does not exist or is not a directory. Did you cloned the repository?"
            ));
        }

        println!("{}", path.to_string_lossy());

        Ok(())
    }
}
