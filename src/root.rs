use std::path::PathBuf;

use anyhow::{anyhow, Result};
use dirs::home_dir;
use tracing::info;

pub struct Root {
    path: PathBuf,
}

impl Root {
    pub fn find() -> Result<Self> {
        let path = home_dir()
            .ok_or_else(|| anyhow!("Could not find a home directory"))?
            .join(".ghr");

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
