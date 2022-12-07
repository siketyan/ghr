use anyhow::Result;
use clap::Parser;
use itertools::Itertools;

use crate::repository::Repositories;
use crate::root::Root;

#[derive(Debug, Parser)]
pub struct Cmd {}

impl Cmd {
    pub fn run(self) -> Result<()> {
        let root = Root::find()?;

        Repositories::try_collect(&root)?
            .into_iter()
            .map(|(path, _)| path.to_string())
            .sorted()
            .for_each(|path| println!("{}", path));

        Ok(())
    }
}
