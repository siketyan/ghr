use std::path::PathBuf;

use anyhow::{Result, bail};
use git2::{BranchType, ErrorCode, Reference, Repository as GitRepository};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::path::Path;

#[derive(Deserialize, Serialize)]
pub struct BranchRef {
    pub name: String,
    pub upstream: String,
}

#[derive(Deserialize, Serialize)]
pub struct RemoteRef {
    pub name: String,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Ref {
    // Internally tagged enums cannot hold a bare string, so the remote ref name is wrapped in a struct.
    #[serde(rename = "remote")]
    Remote(RemoteRef),
    #[serde(rename = "branch")]
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

        let remotes = repo.remotes()?;
        if remotes.is_empty() {
            bail!("No remotes defined");
        }

        let r#ref = match Self::synced_ref(&repo, &head) {
            Ok(r) => Some(r),
            Err(e) => {
                warn!("Repository {} is not synced to remote: {}", path, e);
                None
            }
        };

        Ok(Self {
            host: path.host.to_string(),
            owner: path.owner.to_string(),
            repo: path.repo.to_string(),
            r#ref,
            remotes: remotes
                .iter()
                .filter_map(|name| name.ok().flatten())
                .map(|name| {
                    let remote = repo.find_remote(name)?;

                    Ok(Remote {
                        name: name.to_string(),
                        url: remote.url().unwrap_or_default().to_string(),
                        push_url: remote.pushurl().ok().flatten().map(|u| u.to_string()),
                    })
                })
                .collect::<Result<Vec<_>>>()?,
        })
    }

    fn synced_ref(repo: &GitRepository, head: &Reference) -> Result<Ref> {
        if head.is_remote() {
            return Ok(Ref::Remote(RemoteRef {
                name: head.name().unwrap().to_string(),
            }));
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

            let upstream_name = upstream.name()?.unwrap().to_string();
            let reference = upstream.into_reference();
            if head != &reference {
                bail!("Branch is not synced");
            }

            Ok(Ref::Branch(BranchRef {
                name: name.to_string(),
                upstream: upstream_name,
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
    #[serde(rename = "1")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::root::Root;
    use git2::{Oid, Signature};
    use tempfile::TempDir;

    const HOST: &str = "github.com";
    const OWNER: &str = "siketyan";
    const REPO: &str = "ghr";

    fn setup() -> (TempDir, Root) {
        let dir = tempfile::tempdir().unwrap();
        let root = Root::from_path(dir.path());
        std::fs::create_dir_all(dir.path().join(HOST).join(OWNER).join(REPO)).unwrap();

        (dir, root)
    }

    fn init(path: &Path) -> GitRepository {
        GitRepository::init(PathBuf::from(path)).unwrap()
    }

    fn commit(repo: &GitRepository) -> Oid {
        let sig = Signature::now("ghr", "ghr@example.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let parents = match repo.head() {
            Ok(h) => vec![h.peel_to_commit().unwrap()],
            _ => vec![],
        };

        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "commit",
            &tree,
            &parents.iter().collect::<Vec<_>>(),
        )
        .unwrap()
    }

    fn branch_name(repo: &GitRepository) -> String {
        repo.head().unwrap().shorthand().unwrap().to_string()
    }

    /// Creates `origin/<branch>` pointing at `oid` and sets it as the upstream of the current branch.
    fn set_upstream(repo: &GitRepository, oid: Oid) {
        let branch = branch_name(repo);
        repo.reference(
            &format!("refs/remotes/origin/{}", branch),
            oid,
            true,
            "test",
        )
        .unwrap();

        let mut config = repo.config().unwrap();
        config
            .set_str(&format!("branch.{}.remote", branch), "origin")
            .unwrap();
        config
            .set_str(
                &format!("branch.{}.merge", branch),
                &format!("refs/heads/{}", branch),
            )
            .unwrap();
    }

    #[test]
    fn save_fails_on_non_git_directory() {
        let (_dir, root) = setup();
        let path = Path::new(&root, HOST, OWNER, REPO);

        let err = Repository::save(&path).err().unwrap();
        assert_eq!("Not a Git repository", err.to_string());
    }

    #[test]
    fn save_fails_on_unborn_branch() {
        let (_dir, root) = setup();
        let path = Path::new(&root, HOST, OWNER, REPO);
        init(&path);

        let err = Repository::save(&path).err().unwrap();
        assert_eq!("HEAD is an unborn branch", err.to_string());
    }

    #[test]
    fn save_fails_without_remotes() {
        let (_dir, root) = setup();
        let path = Path::new(&root, HOST, OWNER, REPO);
        let repo = init(&path);
        commit(&repo);

        let err = Repository::save(&path).err().unwrap();
        assert_eq!("No remotes defined", err.to_string());
    }

    #[test]
    fn save_remotes() {
        let (_dir, root) = setup();
        let path = Path::new(&root, HOST, OWNER, REPO);
        let repo = init(&path);
        commit(&repo);
        repo.remote("origin", "https://github.com/siketyan/ghr.git")
            .unwrap();
        repo.remote("upstream", "https://github.com/example/ghr.git")
            .unwrap();
        repo.remote_set_pushurl("upstream", Some("git@github.com:example/ghr.git"))
            .unwrap();

        let saved = Repository::save(&path).unwrap();

        assert_eq!(HOST, saved.host);
        assert_eq!(OWNER, saved.owner);
        assert_eq!(REPO, saved.repo);
        assert_eq!(2, saved.remotes.len());

        let origin = saved.remotes.iter().find(|r| r.name == "origin").unwrap();
        assert_eq!("https://github.com/siketyan/ghr.git", origin.url);
        assert_eq!(None, origin.push_url);

        let upstream = saved.remotes.iter().find(|r| r.name == "upstream").unwrap();
        assert_eq!("https://github.com/example/ghr.git", upstream.url);
        assert_eq!(
            Some("git@github.com:example/ghr.git"),
            upstream.push_url.as_deref(),
        );
    }

    #[test]
    fn save_branch_without_upstream_has_no_ref() {
        let (_dir, root) = setup();
        let path = Path::new(&root, HOST, OWNER, REPO);
        let repo = init(&path);
        commit(&repo);
        repo.remote("origin", "https://github.com/siketyan/ghr.git")
            .unwrap();

        let saved = Repository::save(&path).unwrap();

        assert!(saved.r#ref.is_none());
    }

    #[test]
    fn save_branch_synced_with_upstream() {
        let (_dir, root) = setup();
        let path = Path::new(&root, HOST, OWNER, REPO);
        let repo = init(&path);
        let oid = commit(&repo);
        repo.remote("origin", "https://github.com/siketyan/ghr.git")
            .unwrap();
        set_upstream(&repo, oid);

        let branch = branch_name(&repo);
        let saved = Repository::save(&path).unwrap();

        match saved.r#ref {
            Some(Ref::Branch(b)) => {
                assert_eq!(branch, b.name);
                assert_eq!(format!("origin/{}", branch), b.upstream);
            }
            _ => panic!("expected a branch ref"),
        }
    }

    #[test]
    fn save_branch_ahead_of_upstream_has_no_ref() {
        let (_dir, root) = setup();
        let path = Path::new(&root, HOST, OWNER, REPO);
        let repo = init(&path);
        let oid = commit(&repo);
        repo.remote("origin", "https://github.com/siketyan/ghr.git")
            .unwrap();
        set_upstream(&repo, oid);
        commit(&repo);

        let saved = Repository::save(&path).unwrap();

        assert!(saved.r#ref.is_none());
    }

    #[test]
    fn save_detached_head_has_no_ref() {
        let (_dir, root) = setup();
        let path = Path::new(&root, HOST, OWNER, REPO);
        let repo = init(&path);
        let oid = commit(&repo);
        repo.remote("origin", "https://github.com/siketyan/ghr.git")
            .unwrap();
        repo.set_head_detached(oid).unwrap();

        let saved = Repository::save(&path).unwrap();

        assert!(saved.r#ref.is_none());
    }

    #[test]
    fn file_from_paths_skips_invalid_repositories() {
        let (dir, root) = setup();
        let path = Path::new(&root, HOST, OWNER, REPO);
        let repo = init(&path);
        commit(&repo);
        repo.remote("origin", "https://github.com/siketyan/ghr.git")
            .unwrap();

        std::fs::create_dir_all(dir.path().join(HOST).join(OWNER).join("not-git")).unwrap();

        let file = [
            Path::new(&root, HOST, OWNER, REPO),
            Path::new(&root, HOST, OWNER, "not-git"),
        ]
        .into_iter()
        .collect::<File>();

        assert_eq!(1, file.repositories.len());
        assert_eq!(REPO, file.repositories[0].repo);
    }

    #[test]
    fn file_toml_round_trip() {
        let toml = r#"
version = "1"

[[repositories]]
host = "github.com"
owner = "siketyan"
repo = "ghr"

[repositories.ref]
type = "branch"
name = "main"
upstream = "origin/main"

[[repositories.remotes]]
name = "origin"
url = "https://github.com/siketyan/ghr.git"

[[repositories]]
host = "github.com"
owner = "siketyan"
repo = "other"

"#;

        let file = toml::from_str::<File>(toml).unwrap();
        assert_eq!(2, file.repositories.len());
        assert!(matches!(
            &file.repositories[0].r#ref,
            Some(Ref::Branch(b)) if b.name == "main" && b.upstream == "origin/main",
        ));
        assert_eq!(1, file.repositories[0].remotes.len());
        assert!(file.repositories[1].r#ref.is_none());
        assert!(file.repositories[1].remotes.is_empty());

        let serialized = toml::to_string(&file).unwrap();
        let deserialized = toml::from_str::<File>(&serialized).unwrap();
        assert_eq!(2, deserialized.repositories.len());
        assert_eq!("other", deserialized.repositories[1].repo);
    }

    #[test]
    fn save_head_pointing_at_remote_ref() {
        let (_dir, root) = setup();
        let path = Path::new(&root, HOST, OWNER, REPO);
        let repo = init(&path);
        let oid = commit(&repo);
        repo.remote("origin", "https://github.com/siketyan/ghr.git")
            .unwrap();
        repo.reference("refs/remotes/origin/main", oid, true, "test")
            .unwrap();
        // `set_head` detaches HEAD for non-branch refs, so point HEAD at the remote ref directly.
        repo.reference_symbolic("HEAD", "refs/remotes/origin/main", true, "test")
            .unwrap();

        let saved = Repository::save(&path).unwrap();

        assert!(matches!(
            &saved.r#ref,
            Some(Ref::Remote(r)) if r.name == "refs/remotes/origin/main",
        ));

        let file = File {
            version: Version::V1,
            repositories: vec![saved],
        };
        assert!(toml::to_string(&file).is_ok());
    }

    #[test]
    fn remote_ref_round_trip() {
        let file = File {
            version: Version::V1,
            repositories: vec![Repository {
                host: "github.com".to_string(),
                owner: "siketyan".to_string(),
                repo: "ghr".to_string(),
                r#ref: Some(Ref::Remote(RemoteRef {
                    name: "refs/remotes/origin/main".to_string(),
                })),
                remotes: vec![],
            }],
        };

        let serialized = toml::to_string(&file).unwrap();
        let deserialized = toml::from_str::<File>(&serialized).unwrap();

        assert!(matches!(
            &deserialized.repositories[0].r#ref,
            Some(Ref::Remote(r)) if r.name == "refs/remotes/origin/main",
        ));
    }

    #[test]
    fn deserialize_remote_ref() {
        let file = toml::from_str::<File>(
            r#"
            version = "1"

            [[repositories]]
            host = "github.com"
            owner = "siketyan"
            repo = "ghr"
            ref = { type = "remote", name = "refs/remotes/origin/main" }
            "#,
        )
        .unwrap();

        assert!(matches!(
            &file.repositories[0].r#ref,
            Some(Ref::Remote(r)) if r.name == "refs/remotes/origin/main",
        ));
    }
}
