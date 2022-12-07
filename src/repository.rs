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
