use anyhow::{bail, Result};
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cmd {
    /// URL or pattern of the repository to change directory into.
    repo: String,
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        bail!("Shell extension is not configured correctly.")
    }
}
