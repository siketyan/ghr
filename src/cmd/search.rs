use anyhow::Result;
use clap::Parser;
use itertools::Itertools;
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};

use crate::repository::Repositories;
use crate::root::Root;

const MIN_SCORE: u32 = 50;

#[derive(Debug, Parser)]
pub struct Cmd {
    query: String,
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        let root = Root::find()?;

        let mut matcher = Matcher::new(Config::DEFAULT);
        let pattern = Pattern::new(
            &self.query,
            CaseMatching::Smart,
            Normalization::Smart,
            AtomKind::Fuzzy,
        );

        let matches = pattern.match_list(
            Repositories::try_collect(&root)?
                .into_iter()
                .map(|(path, _)| path.to_string()),
            &mut matcher,
        );

        matches
            .into_iter()
            .filter(|(_, score)| *score > MIN_SCORE)
            .sorted_by_key(|(_, score)| -i64::from(*score))
            .for_each(|(path, _)| println!("{}", path));

        Ok(())
    }
}
