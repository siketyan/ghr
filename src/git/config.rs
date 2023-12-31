use serde::Deserialize;

use crate::git::strategy::Strategy;

#[derive(Debug, Default, Deserialize)]
pub struct StrategyConfig {
    #[serde(default)]
    pub clone: Strategy,
    #[serde(default)]
    pub fetch: Strategy,
    #[serde(default)]
    pub checkout: Strategy,
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub strategy: StrategyConfig,
}
