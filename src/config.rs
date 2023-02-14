use std::fs::read_to_string;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

use crate::application::Applications;
use crate::git::Config as GitConfig;
use crate::profile::Profiles;
use crate::root::Root;
use crate::rule::Rules;
use crate::url::Patterns;

#[derive(Debug, Default, Deserialize)]
pub struct Defaults {
    pub owner: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub patterns: Patterns,
    #[serde(default)]
    pub profiles: Profiles,
    #[serde(default)]
    pub applications: Applications,
    #[serde(default)]
    pub rules: Rules,
}

impl Config {
    pub fn load_from(root: &Root) -> Result<Self> {
        Ok(Self::load_from_path(root.path().join("ghr.toml"))?.unwrap_or_default())
    }

    pub fn load() -> Result<Self> {
        Self::load_from(&Root::find()?)
    }

    fn load_from_path<P>(path: P) -> Result<Option<Self>>
    where
        P: AsRef<Path>,
    {
        Ok(match path.as_ref().exists() {
            true => Some(Self::load_from_str(read_to_string(path)?.as_str())?),
            _ => None,
        })
    }

    fn load_from_str(s: &str) -> Result<Self> {
        Ok(toml::from_str(s)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;

    #[test]
    fn load_example_config() {
        Config::load_from_str(include_str!("../config.example.toml")).unwrap();
    }
}
