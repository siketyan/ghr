use anyhow::Result;
use clap::Parser;

use crate::BUILD_INFO;

#[derive(Debug, Parser)]
pub struct Cmd {}

impl Cmd {
    pub fn run(self) -> Result<()> {
        println!("{}", BUILD_INFO);

        Ok(())
    }
}
