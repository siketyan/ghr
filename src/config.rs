use std::fs::read_to_string;

use anyhow::Result;
use serde::Deserialize;

use crate::application::Applications;
use crate::profile::Profiles;
use crate::root::Root;
use crate::rule::Rules;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub profiles: Profiles,
    #[serde(default)]
    pub applications: Applications,
    #[serde(default)]
    pub rules: Rules,
}

impl Config {
    pub fn load_from(root: &Root) -> Result<Self> {
        let path = root.path().join("config.toml");

        Ok(match path.exists() {
            true => toml::from_str(read_to_string(path)?.as_str())?,
            _ => Self::default(),
        })
    }

    pub fn load() -> Result<Self> {
        Self::load_from(&Root::find()?)
    }
}
