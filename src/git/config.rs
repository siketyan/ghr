use serde::Deserialize;

use crate::git::strategy::Strategy;

#[derive(Debug, Default, Deserialize)]
pub struct StrategyConfig {
    #[serde(default)]
    pub clone: Strategy,
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub strategy: StrategyConfig,
}
