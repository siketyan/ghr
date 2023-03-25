use anyhow::{anyhow, Result};
use async_trait::async_trait;
use gh_config::{Hosts, GITHUB_COM};
use octocrab::Octocrab;
use serde::Deserialize;

use crate::platform::{Fork, Platform, PlatformInit};
use crate::url::Url;

fn default_host() -> String {
    GITHUB_COM.to_string()
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub(super) host: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: default_host(),
        }
    }
}

pub struct GitHub {
    client: Octocrab,
}

impl PlatformInit for GitHub {
    type Config = Config;

    fn init(config: &Config) -> Result<Self> {
        let token = Hosts::load()?
            .get(&config.host)
            .ok_or_else(|| {
                anyhow!(
                    "gh CLI does not have any token for github.com. Run `gh auth login` and retry."
                )
            })?
            .oauth_token
            .clone();

        let mut builder = Octocrab::builder().personal_token(token);
        if config.host != GITHUB_COM {
            builder = builder.base_url(format!("https://{}/api/v3", &config.host))?;
        }

        Ok(Self {
            client: builder.build()?,
        })
    }
}

impl Platform for GitHub {}

#[async_trait]
impl Fork for GitHub {
    async fn fork(&self, url: &Url, owner: Option<String>) -> Result<String> {
        let request = self.client.repos(&url.owner, &url.repo);
        let request = match owner {
            Some(o) => request.create_fork().organization(o),
            _ => request.create_fork(),
        };

        Ok(request
            .send()
            .await?
            .html_url
            .as_ref()
            .ok_or_else(|| anyhow!("GitHub API did not return HTML URL for the repository."))?
            .to_string())
    }
}
