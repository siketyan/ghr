use std::fs::read_to_string;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;
use tracing::warn;

use crate::application::Applications;
use crate::platform::Config as PlatformConfig;
use crate::profile::Profiles;
use crate::root::Root;
use crate::rule::Rules;
use crate::url::Patterns;

#[derive(Debug, Default, Deserialize)]
pub struct Defaults {
    pub owner: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct SearchPath {
    #[serde(default)]
    pub owner: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    #[deprecated(since = "0.5.0", note = "Use search_path instead.")]
    pub defaults: Defaults,
    #[serde(default)]
    pub search_path: SearchPath,
    #[serde(default)]
    pub platforms: PlatformConfig,
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
        let mut config = toml::from_str::<Self>(s)?;
        #[allow(deprecated)]
        if let Some(default_owner) = &config.defaults.owner {
            warn!("Section [defaults] in config is deprecated. Use [search_path] instead.");

            config
                .search_path
                .owner
                .insert(0, default_owner.to_string());
        }

        Ok(config.with_defaults())
    }

    fn with_defaults(mut self) -> Self {
        self.patterns = self.patterns.with_defaults();
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;

    #[test]
    fn load_example_config() {
        Config::load_from_str(include_str!("../ghr.example.toml")).unwrap();
    }
}
