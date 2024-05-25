use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use gh_config::{is_enterprise, retrieve_token_from_env, retrieve_token_secure, Hosts, GITHUB_COM};
use octocrab::Octocrab;
use serde::Deserialize;

use crate::platform::{Browse, Fork, Platform, PlatformInit};
use crate::url::Url;

fn default_host() -> String {
    GITHUB_COM.to_string()
}

#[derive(Debug, Deserialize, Clone)]
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
    config: Config,
}

impl PlatformInit for GitHub {
    type Config = Config;

    fn init(config: &Config) -> Result<Self> {
        // Try to load ~/.config/gh/hosts.yml if exists.
        let hosts = match Hosts::load() {
            Ok(h) => Some(h),
            Err(gh_config::Error::Io(_)) => None,
            Err(e) => return Err(e).context("Could not read the configuration of gh CLI."),
        };

        let host = config.host.as_str();
        let token = match hosts {
            // If the hosts.yml exists, retrieve token from the env, hosts.yml, or secure storage.
            Some(h) => h.retrieve_token(host)?,
            // Otherwise, retrieve token from the env or secure storage, skipping hosts.yml.
            _ => match retrieve_token_from_env(is_enterprise(host)) {
                Some(t) => Some(t),
                _ => retrieve_token_secure(host)?,
            },
        };

        let token = match token {
            Some(t) => t,
            _ => bail!("GitHub access token could not be found. Install the gh CLI and login, or provide an token as GH_TOKEN environment variable."),
        };

        let mut builder = Octocrab::builder().personal_token(token);
        if config.host != GITHUB_COM {
            builder = builder.base_uri(format!("https://{}/api/v3", &config.host))?;
        }

        Ok(Self {
            client: builder.build()?,
            config: config.clone(),
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

#[async_trait]
impl Browse for GitHub {
    async fn get_browsable_url(&self, url: &Url) -> Result<url::Url> {
        Ok(url::Url::parse(&format!(
            "https://{}/{}/{}",
            self.config.host, url.owner, url.repo
        ))?)
    }
}
