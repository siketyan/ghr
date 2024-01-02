use anyhow::Result;
use clap::Parser;

use crate::{BUILD_INFO, VERSION};

#[derive(Debug, Parser)]
pub struct Cmd {
    #[clap(short, long)]
    short: bool,
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        println!(
            "{}",
            match self.short {
                true => VERSION,
                _ => BUILD_INFO,
            },
        );

        Ok(())
    }
}
