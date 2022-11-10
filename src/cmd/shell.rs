use std::fmt::{Display, Formatter};

use anyhow::Result;
use clap::{Parser, ValueEnum};

#[derive(Clone, Debug, Default, ValueEnum)]
pub enum Kind {
    #[default]
    Bash,
    Fish,
}

impl Display for Kind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Kind::Bash => "bash",
                Kind::Fish => "fish",
            },
        )
    }
}

#[derive(Debug, Parser)]
pub struct Cmd {
    /// Kind of the shell.
    #[clap(default_value_t)]
    kind: Kind,
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        let script = match self.kind {
            Kind::Bash => include_str!("../../resources/shell/ghr.bash"),
            Kind::Fish => include_str!("../../resources/shell/ghr.fish"),
        };

        print!("{}", script);

        Ok(())
    }
}
