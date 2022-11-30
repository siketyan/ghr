use std::fs::read_to_string;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;
use tracing::warn;

use crate::application::Applications;
use crate::git::Config as GitConfig;
use crate::profile::Profiles;
use crate::root::Root;
use crate::rule::Rules;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub profiles: Profiles,
    #[serde(default)]
    pub applications: Applications,
    #[serde(default)]
    pub rules: Rules,
}

impl Config {
    pub fn load_from(root: &Root) -> Result<Self> {
        Ok(match Self::load_from_path(root.path().join("ghr.toml"))? {
            Some(c) => c,
            None => {
                warn!(
                    "Using `config.toml` is deprecated since ghr v0.2.3 and \
                    it won't be supported from v0.3. Move them to `ghr.toml` to migrate.",
                );

                Self::load_from_path(root.path().join("config.toml"))?.unwrap_or_default()
            }
        })
    }

    pub fn load() -> Result<Self> {
        Self::load_from(&Root::find()?)
    }

    fn load_from_path<P>(path: P) -> Result<Option<Self>>
    where
        P: AsRef<Path>,
    {
        Ok(match path.as_ref().exists() {
            true => Some(toml::from_str(read_to_string(path)?.as_str())?),
            _ => None,
        })
    }
}
