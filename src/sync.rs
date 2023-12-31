use std::path::PathBuf;

use anyhow::{bail, Result};
use git2::{BranchType, ErrorCode, Reference, Repository as GitRepository};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::path::Path;

#[derive(Deserialize, Serialize)]
pub enum Ref {}

#[derive(Deserialize, Serialize)]
pub struct Remote {
    pub name: String,
    pub url: String,
    pub push_url: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Repository {
    pub host: String,
    pub owner: String,
    pub repo: String,
    pub r#ref: String,
    #[serde(default)]
    pub remotes: Vec<Remote>,
}

impl Repository {
    pub fn save(path: &Path) -> Result<Self> {
        let repo = match GitRepository::open(PathBuf::from(path)) {
            Ok(r) => r,
            Err(e) => match e.code() {
                ErrorCode::NotFound => bail!("Not a Git repository"),
                _ => return Err(e.into()),
            },
        };

        let head = match repo.head() {
            Ok(r) => r,
            Err(e) => match e.code() {
                ErrorCode::UnbornBranch => bail!("HEAD is an unborn branch"),
                ErrorCode::NotFound => bail!("Cannot find the HEAD"),
                _ => return Err(e.into()),
            },
        };

        if let Err(e) = Self::ensure_synced(&repo, &head) {
            warn!("Repository {} is not synced to remote: {}", path, e);
        }

        let r#ref = head.name().unwrap_or_default().to_string();

        Ok(Self {
            host: path.host.to_string(),
            owner: path.owner.to_string(),
            repo: path.repo.to_string(),
            r#ref,
            remotes: repo
                .remotes()?
                .iter()
                .flatten()
                .map(|name| {
                    let remote = repo.find_remote(name)?;

                    Ok(Remote {
                        name: name.to_string(),
                        url: remote.url().unwrap_or_default().to_string(),
                        push_url: remote.pushurl().map(|u| u.to_string()),
                    })
                })
                .collect::<Result<Vec<_>>>()?,
        })
    }

    fn ensure_synced(repo: &GitRepository, head: &Reference) -> Result<()> {
        if head.is_remote() {
            return Ok(());
        }

        if head.is_branch() {
            let upstream = match repo
                .find_branch(head.shorthand().unwrap(), BranchType::Local)?
                .upstream()
            {
                Ok(b) => b,
                Err(e) => match e.code() {
                    ErrorCode::NotFound => bail!("Branch has never pushed to remote"),
                    _ => return Err(e.into()),
                },
            };

            if head != &upstream.into_reference() {
                bail!("Branch is not synced");
            }
        } else if head.is_tag() {
            bail!("HEAD is a tag");
        } else {
            bail!("Detached HEAD");
        }

        return Ok(());
    }
}

#[derive(Deserialize, Serialize)]
pub struct File {
    #[serde(default)]
    pub repositories: Vec<Repository>,
}

impl<'a> FromIterator<Path<'a>> for File {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Path<'a>>,
    {
        Self {
            repositories: iter
                .into_iter()
                .flat_map(|path| match Repository::save(&path) {
                    Ok(r) => Some(r),
                    Err(e) => {
                        warn!("Skipped repository {}: {}", &path, e);
                        None
                    }
                })
                .collect::<Vec<_>>(),
        }
    }
}
