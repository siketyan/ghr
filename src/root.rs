use std::env::var;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use dirs::home_dir;
use tracing::info;

const ENV_VAR_KEY: &str = "GHR_ROOT";
const DEFAULT_ROOT_NAME: &str = ".ghr";

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

        info!(
            "Found a root directory: {}",
            path.to_str().unwrap_or_default(),
        );

        Ok(Self { path })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}
