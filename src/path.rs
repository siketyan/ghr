use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::root::Root;
use crate::url::Url;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Path<'a> {
    root: &'a Root,
    pub host: String,
    pub owner: String,
    pub repo: String,
}

impl<'a> Path<'a> {
    pub fn new(
        root: &'a Root,
        host: impl Into<String>,
        owner: impl Into<String>,
        repo: impl Into<String>,
    ) -> Self {
        Self {
            root,
            host: host.into(),
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    pub fn resolve(root: &'a Root, url: &Url) -> Self {
        Self {
            root,
            host: url.host.to_string(),
            owner: url.owner.clone(),
            repo: url.repo.clone(),
        }
    }

    pub fn to_string_with(&self, host: bool, owner: bool) -> String {
        match (host, owner) {
            (false, true) => format!("{}/{}", self.owner, self.repo),
            (true, false) => format!("{}:{}", self.host, self.repo),
            (false, false) => self.repo.to_string(),
            _ => format!("{}:{}/{}", self.host, self.owner, self.repo),
        }
    }
}

impl Display for Path<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}/{}", self.host, self.owner, self.repo)
    }
}

impl<'a> From<&Path<'a>> for PathBuf {
    fn from(p: &Path<'a>) -> Self {
        p.root.path().join(&p.host).join(&p.owner).join(&p.repo)
    }
}

impl<'a> From<Path<'a>> for PathBuf {
    fn from(p: Path<'a>) -> Self {
        (&p).into()
    }
}

pub struct PartialPath<'a> {
    pub root: &'a Root,
    pub host: Option<String>,
    pub owner: Option<String>,
    pub repo: Option<String>,
}

impl<'a> From<&PartialPath<'a>> for PathBuf {
    fn from(p: &PartialPath<'a>) -> Self {
        let mut path = p.root.path().to_owned();

        match p.host.as_deref() {
            Some(h) => path = path.join(h),
            _ => return path,
        }

        match p.owner.as_deref() {
            Some(o) => path = path.join(o),
            _ => return path,
        }

        match p.repo.as_deref() {
            Some(r) => path.join(r),
            _ => path,
        }
    }
}

impl<'a> From<PartialPath<'a>> for PathBuf {
    fn from(p: PartialPath<'a>) -> Self {
        (&p).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::url::{Host, Patterns};

    fn root() -> Root {
        Root::from_path("/ghr")
    }

    #[test]
    fn to_string_with_all_combinations() {
        let root = root();
        let path = Path::new(&root, "github.com", "siketyan", "ghr");

        assert_eq!("github.com:siketyan/ghr", path.to_string_with(true, true));
        assert_eq!("siketyan/ghr", path.to_string_with(false, true));
        assert_eq!("github.com:ghr", path.to_string_with(true, false));
        assert_eq!("ghr", path.to_string_with(false, false));
    }

    #[test]
    fn display() {
        let root = root();
        let path = Path::new(&root, "example.com", "owner", "repo");

        assert_eq!("example.com:owner/repo", path.to_string());
    }

    #[test]
    fn resolve_from_url() {
        let root = root();
        let url = Url::from_str("siketyan/ghr", &Patterns::default(), None).unwrap();
        let path = Path::resolve(&root, &url);

        assert_eq!(Host::GitHub.to_string(), path.host);
        assert_eq!("siketyan", path.owner);
        assert_eq!("ghr", path.repo);
    }

    #[test]
    fn into_path_buf() {
        let root = root();
        let path = Path::new(&root, "github.com", "siketyan", "ghr");

        assert_eq!(
            PathBuf::from("/ghr/github.com/siketyan/ghr"),
            PathBuf::from(&path),
        );
        assert_eq!(
            PathBuf::from("/ghr/github.com/siketyan/ghr"),
            PathBuf::from(path),
        );
    }

    #[test]
    fn partial_path_into_path_buf() {
        let root = root();
        let partial = |host: Option<&str>, owner: Option<&str>, repo: Option<&str>| PartialPath {
            root: &root,
            host: host.map(String::from),
            owner: owner.map(String::from),
            repo: repo.map(String::from),
        };

        assert_eq!(
            PathBuf::from("/ghr"),
            PathBuf::from(partial(None, None, None))
        );
        assert_eq!(
            PathBuf::from("/ghr/github.com"),
            PathBuf::from(partial(Some("github.com"), None, None)),
        );
        assert_eq!(
            PathBuf::from("/ghr/github.com/siketyan"),
            PathBuf::from(partial(Some("github.com"), Some("siketyan"), None)),
        );
        assert_eq!(
            PathBuf::from("/ghr/github.com/siketyan/ghr"),
            PathBuf::from(&partial(Some("github.com"), Some("siketyan"), Some("ghr"))),
        );
    }

    #[test]
    fn partial_path_stops_at_first_missing_segment() {
        let root = root();
        let partial = PartialPath {
            root: &root,
            host: None,
            owner: Some("siketyan".to_string()),
            repo: Some("ghr".to_string()),
        };

        assert_eq!(PathBuf::from("/ghr"), PathBuf::from(partial));
    }
}
