use std::io::{read_to_string, stdin};
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use git2::{ErrorCode, Repository as GitRepository};
use tracing::info;

use crate::cmd::clone;
use crate::config::Config;
use crate::path::Path;
use crate::root::Root;
use crate::sync::{File, Ref, Repository};
use crate::url::Url;

#[derive(Debug, Parser)]
pub struct Cmd {}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        let root = Root::find()?;
        let config = Config::load_from(&root)?;
        let file = toml::from_str::<File>(read_to_string(stdin())?.as_str())?;

        for Repository {
            host,
            owner,
            repo,
            r#ref,
            remotes,
        } in file.repositories
        {
            let origin = remotes.iter().find(|r| {
                Url::from_str(&r.url, &config.patterns, config.defaults.owner.as_deref())
                    .ok()
                    .map(|u| u.host.to_string() == host)
                    .unwrap_or_default()
            });

            clone::Cmd {
                repo: vec![origin
                    .map(|r| r.url.to_string())
                    .unwrap_or_else(|| format!("{}:{}/{}", host, owner, repo))],
                origin: origin.map(|r| r.name.to_string()),
                ..Default::default()
            }
            .run()
            .await?;

            let path = Path::new(&root, host, owner, repo);
            let repo = GitRepository::open(PathBuf::from(&path))?;

            for remote in remotes {
                if let Err(e) = repo
                    .remote(&remote.name, &remote.url)
                    .and_then(|_| repo.remote_set_pushurl(&remote.name, remote.push_url.as_deref()))
                {
                    match e.code() {
                        ErrorCode::Exists => (),
                        _ => return Err(e.into()),
                    }
                }
            }

            match r#ref {
                Some(Ref::Remote(r)) => {
                    repo.checkout_tree(&repo.revparse_single(&r)?, None)?;

                    info!("Successfully checked out a remote ref: {}", &r);
                }
                Some(Ref::Branch(b)) => {
                    let (object, reference) = repo.revparse_ext(&b.name)?;
                    let ref_name = reference.unwrap().name().unwrap().to_string();

                    repo.checkout_tree(&object, None)?;
                    repo.set_head(&ref_name)?;

                    info!("Successfully checked out a branch: {}", &ref_name);
                }
                _ => (),
            }
        }

        Ok(())
    }
}
