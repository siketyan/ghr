use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use itertools::Itertools;
use walkdir::WalkDir;

use crate::path::Path;
use crate::root::Root;

pub struct Repository {
    #[allow(dead_code)]
    path: PathBuf,
}

impl Repository {
    fn new<P>(path: P) -> Self
    where
        P: AsRef<std::path::Path>,
    {
        Self {
            path: PathBuf::from(path.as_ref()),
        }
    }
}

pub struct Repositories<'a> {
    map: HashMap<Path<'a>, Repository>,
}

impl<'a> Repositories<'a> {
    pub fn try_collect(root: &'a Root) -> Result<Self> {
        Ok(Self {
            map: WalkDir::new(root.path())
                .min_depth(3)
                .max_depth(3)
                .into_iter()
                .map_ok(|entry| entry.into_path())
                .filter_ok(|path| path.is_dir())
                .map_ok(|path| {
                    let parts = path.strip_prefix(root.path())?.iter().collect::<Vec<_>>();

                    Ok::<_, anyhow::Error>((
                        Path::new(
                            root,
                            parts[0].to_string_lossy(),
                            parts[1].to_string_lossy(),
                            parts[2].to_string_lossy(),
                        ),
                        Repository::new(path),
                    ))
                })
                .flatten()
                .try_collect()?,
        })
    }
}

impl<'a> IntoIterator for Repositories<'a> {
    type Item = (Path<'a>, Repository);
    type IntoIter = std::collections::hash_map::IntoIter<Path<'a>, Repository>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, write};

    #[test]
    fn collect_repositories_at_depth_three() {
        let dir = tempfile::tempdir().unwrap();
        let root = Root::from_path(dir.path());

        create_dir_all(dir.path().join("github.com/siketyan/ghr")).unwrap();
        create_dir_all(dir.path().join("github.com/siketyan/other/nested")).unwrap();
        create_dir_all(dir.path().join("example.com/owner/repo")).unwrap();
        create_dir_all(dir.path().join("github.com/empty-owner")).unwrap();
        write(dir.path().join("github.com/siketyan/not-a-dir"), "").unwrap();
        write(dir.path().join("ghr.toml"), "").unwrap();

        let paths = Repositories::try_collect(&root)
            .unwrap()
            .into_iter()
            .map(|(p, _)| p.to_string())
            .sorted()
            .collect::<Vec<_>>();

        assert_eq!(
            vec![
                "example.com:owner/repo",
                "github.com:siketyan/ghr",
                "github.com:siketyan/other",
            ],
            paths,
        );
    }

    #[test]
    fn collect_from_empty_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = Root::from_path(dir.path());

        assert_eq!(
            0,
            Repositories::try_collect(&root)
                .unwrap()
                .into_iter()
                .count()
        );
    }
}
