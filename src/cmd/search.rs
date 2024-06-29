use anyhow::Result;
use clap::Parser;
use itertools::Itertools;
use nucleo_matcher::{Config, Matcher, Utf32Str};

use crate::repository::Repositories;
use crate::root::Root;

const MIN_SCORE: u16 = 50;

#[derive(Debug, Parser)]
pub struct Cmd {
    query: String,
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        let root = Root::find()?;

        let mut matcher = Matcher::new(Config::DEFAULT);

        Repositories::try_collect(&root)?
            .into_iter()
            .map(|(path, _)| path.to_string())
            .filter_map(|path| {
                let mut haystack_buf = Vec::new();
                let mut needle_buf = Vec::new();

                matcher
                    .fuzzy_match(
                        Utf32Str::new(&path, &mut haystack_buf),
                        Utf32Str::new(&self.query, &mut needle_buf),
                    )
                    .map(|score| (path, score))
            })
            .filter(|(_, score)| *score > MIN_SCORE)
            .sorted_by_key(|(_, score)| -i32::from(*score))
            .for_each(|(path, _)| println!("{}", path));

        Ok(())
    }
}
