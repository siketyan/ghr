mod list;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Action {
    List(list::Cmd),
}

#[derive(Debug, Parser)]
pub struct Cmd {
    #[clap(subcommand)]
    action: Action,
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        use Action::*;
        match self.action {
            List(cmd) => cmd.run(),
        }
    }
}
