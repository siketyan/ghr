use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Application {
    cmd: String,
    args: Vec<String>,
}

impl Application {
    pub fn intermediate(cmd: &str) -> Self {
        Self {
            cmd: cmd.to_string(),
            args: vec!["%p".to_string()],
        }
    }

    pub fn open<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let _ = Command::new(&self.cmd)
            .args(
                self.args
                    .iter()
                    .map(|arg| match arg.as_str() {
                        "%p" => path.as_ref().to_string_lossy().to_string(),
                        _ => arg.to_string(),
                    })
                    .collect::<Vec<_>>(),
            )
            .spawn()?;

        Ok(())
    }
}

#[cfg(windows)]
impl Default for Application {
    fn default() -> Self {
        Self {
            cmd: "explorer.exe".to_string(),
            args: vec!["%p".to_string()],
        }
    }
}

#[cfg(not(windows))]
impl Default for Application {
    fn default() -> Self {
        Self {
            cmd: "open".to_string(),
            args: vec!["%p".to_string()],
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct Applications {
    #[serde(flatten)]
    map: HashMap<String, Application>,
}

impl Applications {
    pub fn open<P>(&self, name: &str, path: P) -> Option<Result<()>>
    where
        P: AsRef<Path>,
    {
        self.get(name).map(|a| a.open(path))
    }

    pub fn open_or_intermediate<P>(&self, name: &str, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        self.open(name, &path)
            .unwrap_or_else(|| Application::intermediate(name).open(&path))
    }

    pub fn open_or_intermediate_or_default<P>(&self, name: Option<&str>, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        match name {
            Some(n) => self.open_or_intermediate(n, path),
            _ => Application::default().open(path),
        }
    }
}

impl Deref for Applications {
    type Target = HashMap<String, Application>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}
