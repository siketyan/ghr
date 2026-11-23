use std::fs::read_to_string;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

use crate::application::Applications;
use crate::git::Config as GitConfig;
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
pub struct Config {
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub git: GitConfig,
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
        Ok(toml::from_str::<Self>(s)?.with_defaults())
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

    use crate::root::Root;
    use crate::url::Url;

    #[test]
    fn load_example_config_values() {
        let config = Config::load_from_str(include_str!("../ghr.example.toml")).unwrap();

        assert_eq!(Some("siketyan"), config.defaults.owner.as_deref());
        assert!(config.profiles.contains_key("work"));
        assert!(config.applications.contains_key("vscode"));

        // Custom patterns come before the defaults, and the defaults are still available.
        let url = Url::from_str(
            "https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git",
            &config.patterns,
            None,
        )
        .unwrap();
        assert_eq!("torvalds", url.owner);
        assert_eq!("linux", url.repo);

        let url = Url::from_str("siketyan/ghr", &config.patterns, None).unwrap();
        assert_eq!("siketyan", url.owner);
        assert_eq!("ghr", url.repo);
    }

    #[test]
    fn load_empty_config() {
        let config = Config::load_from_str("").unwrap();

        assert!(config.defaults.owner.is_none());
        assert!(config.profiles.is_empty());
        assert!(config.applications.is_empty());
        assert!(Url::from_str("siketyan/ghr", &config.patterns, None).is_ok());
    }

    #[test]
    fn load_invalid_config() {
        assert!(Config::load_from_str("defaults = 1").is_err());
    }

    #[test]
    fn load_from_root_without_file() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::load_from(&Root::from_path(dir.path())).unwrap();

        assert!(config.defaults.owner.is_none());
        assert!(Url::from_str("siketyan/ghr", &config.patterns, None).is_ok());
    }

    #[test]
    fn load_from_root_with_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("ghr.toml"),
            "[defaults]\nowner = \"siketyan\"\n",
        )
        .unwrap();

        let config = Config::load_from(&Root::from_path(dir.path())).unwrap();

        assert_eq!(Some("siketyan"), config.defaults.owner.as_deref());
    }
}
