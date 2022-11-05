use crate::path::PartialPath;
use crate::root::Root;
use anyhow::{anyhow, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Cmd {
    /// Remote host of the repository.
    /// Defaults to github.com.
    #[clap(long)]
    host: Option<String>,
    /// Owner name of the repository.
    owner: Option<String>,
    /// Repository name.
    repo: Option<String>,
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        let root = Root::find()?;
        let path = PartialPath {
            root: &root,
            host: self.host,
            owner: self.owner,
            repo: self.repo,
        };

        let path = PathBuf::from(path);
        if !path.exists() || !path.is_dir() {
            return Err(anyhow!(
                "The path does not exist or is not a directory. Did you cloned the repository?"
            ));
        }

        println!("{}", path.to_string_lossy());

        Ok(())
    }
}
