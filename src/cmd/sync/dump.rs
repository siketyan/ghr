use anyhow::Result;
use clap::Parser;

use crate::repository::Repositories;
use crate::root::Root;
use crate::sync::File;

#[derive(Debug, Parser)]
pub struct Cmd {}

impl Cmd {
    pub fn run(self) -> Result<()> {
        let root = Root::find()?;
        let file = Repositories::try_collect(&root)?
            .into_iter()
            .map(|(p, _)| p)
            .collect::<File>();

        println!("{}", toml::to_string(&file)?);

        Ok(())
    }
}
