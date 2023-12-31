use std::path::PathBuf;

use anyhow::{bail, Result};
use git2::{BranchType, ErrorCode, Reference, Repository as GitRepository};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::path::Path;

#[derive(Deserialize, Serialize)]
pub struct BranchRef {
    pub name: String,
    pub remote: String,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Ref {
    Remote(String),
    Branch(BranchRef),
    // tag is not supported ... yet.
}

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
    pub r#ref: Option<Ref>,
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

        let r#ref = match Self::synced_ref(&repo, &head) {
            Ok(r) => Some(r),
            Err(e) => {
                warn!("Repository {} is not synced to remote: {}", path, e);
                None
            }
        };

        let remotes = repo.remotes()?;
        if remotes.is_empty() {
            bail!("No remotes defined");
        }

        Ok(Self {
            host: path.host.to_string(),
            owner: path.owner.to_string(),
            repo: path.repo.to_string(),
            r#ref,
            remotes: remotes
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

    fn synced_ref(repo: &GitRepository, head: &Reference) -> Result<Ref> {
        if head.is_remote() {
            return Ok(Ref::Remote(head.name().unwrap().to_string()));
        }

        if head.is_branch() {
            let name = head.shorthand().unwrap();
            let upstream = match repo.find_branch(name, BranchType::Local)?.upstream() {
                Ok(b) => b,
                Err(e) => match e.code() {
                    ErrorCode::NotFound => bail!("Branch has never pushed to remote"),
                    _ => return Err(e.into()),
                },
            };

            let reference = upstream.into_reference();
            if head != &reference {
                bail!("Branch is not synced");
            }

            Ok(Ref::Branch(BranchRef {
                name: name.to_string(),
                remote: repo
                    .branch_remote_name(reference.name().unwrap())?
                    .as_str()
                    .unwrap()
                    .to_string(),
            }))
        } else if head.is_tag() {
            bail!("HEAD is a tag");
        } else {
            bail!("Detached HEAD");
        }
    }
}

#[derive(Deserialize, Serialize)]
pub enum Version {
    V1,
}

#[derive(Deserialize, Serialize)]
pub struct File {
    pub version: Version,

    #[serde(default)]
    pub repositories: Vec<Repository>,
}

impl<'a> FromIterator<Path<'a>> for File {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Path<'a>>,
    {
        Self {
            version: Version::V1,
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
