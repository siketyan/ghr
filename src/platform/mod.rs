mod github;

use std::result::Result as StdResult;

use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;

use crate::url::Url;

#[async_trait]
pub trait Fork {
    async fn fork(&self, url: &Url, owner: Option<String>) -> Result<String>;
}

#[async_trait]
pub trait Browse {
    async fn get_browsable_url(&self, url: &Url) -> Result<url::Url>;
}

pub trait PlatformInit: Sized {
    type Config;

    fn init(config: &Self::Config) -> Result<Self>;
}

pub trait Platform: Fork + Browse {}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum PlatformConfig {
    #[cfg(feature = "github")]
    #[serde(rename = "github")]
    GitHub(github::Config),
}

impl PlatformConfig {
    pub fn try_into_platform(&self) -> Result<Box<dyn Platform>> {
        self.try_into()
    }

    fn host(&self) -> String {
        match self {
            #[cfg(feature = "github")]
            Self::GitHub(c) => c.host.to_string(),
        }
    }
}

impl TryInto<Box<dyn Platform>> for &PlatformConfig {
    type Error = anyhow::Error;

    fn try_into(self) -> StdResult<Box<dyn Platform>, Self::Error> {
        Ok(match self {
            #[cfg(feature = "github")]
            PlatformConfig::GitHub(c) => Box::new(github::GitHub::init(c)?),
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    map: HashMap<String, PlatformConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            map: HashMap::from([
                #[cfg(feature = "github")]
                (
                    "github".to_string(),
                    PlatformConfig::GitHub(github::Config::default()),
                ),
            ]),
        }
    }
}

impl Config {
    pub fn find(&self, url: &Url) -> Option<&PlatformConfig> {
        self.map.values().find(|c| c.host() == url.host.to_string())
    }
}
