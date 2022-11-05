use crate::root::Root;
use crate::url::Url;
use std::path::PathBuf;

pub struct Path<'a> {
    root: &'a Root,
    host: String,
    owner: String,
    repo: String,
}

impl<'a> Path<'a> {
    pub fn resolve(root: &'a Root, url: &Url) -> Self {
        Self {
            root,
            host: url.host.to_string(),
            owner: url.owner.clone(),
            repo: url.repo.clone(),
        }
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
