use anyhow::Result;
use clap::Parser;
use itertools::Itertools;
use std::path::PathBuf;

use crate::repository::Repositories;
use crate::root::Root;

#[derive(Debug, Parser)]
pub struct Cmd {
    /// Lists repositories without their hosts.
    #[clap(long)]
    no_host: bool,

    /// Lists repositories without their owners.
    #[clap(long)]
    no_owner: bool,

    /// Lists repositories as full paths instead of their names.
    #[clap(short, long)]
    path: bool,
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        let root = Root::find()?;

        Repositories::try_collect(&root)?
            .into_iter()
            .map(|(path, _)| match self.path {
                true => PathBuf::from(path).to_string_lossy().to_string(),
                _ => path.to_string_with(!self.no_host, !self.no_owner),
            })
            .sorted()
            .for_each(|path| println!("{}", path));

        Ok(())
    }
}
