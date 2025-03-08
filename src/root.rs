use std::env::var;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{Result, anyhow};
use dirs::home_dir;
use tracing::debug;

const ENV_VAR_KEY: &str = "GHR_ROOT";
const DEFAULT_ROOT_NAME: &str = ".ghr";

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Root {
    path: PathBuf,
}

impl Root {
    pub fn find() -> Result<Self> {
        let path = match var(ENV_VAR_KEY).ok().and_then(|s| match s.is_empty() {
            true => None,
            _ => Some(s),
        }) {
            Some(p) => PathBuf::from_str(&p)?.canonicalize()?,
            _ => home_dir()
                .ok_or_else(|| anyhow!("Could not find a home directory"))?
                .join(DEFAULT_ROOT_NAME),
        };

        debug!(
            "Found a root directory: {}",
            path.to_str().unwrap_or_default(),
        );

        Ok(Self { path })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}
