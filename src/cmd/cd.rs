use anyhow::{bail, Result};
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cmd {
    /// URL or pattern of the repository where to change directory to.
    repo: String,
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        bail!("Shell extension is not configured correctly.")
    }
}
