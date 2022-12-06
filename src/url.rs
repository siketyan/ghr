use std::convert::Infallible;
use std::str::FromStr;

use anyhow::{anyhow, bail, Error, Result};
use itertools::FoldWhile;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::{Captures, Regex};

const GITHUB_COM: &str = "github.com";

const GIT_EXTENSION: &str = ".git";
const EXTENSIONS: &[&str] = &[GIT_EXTENSION];

#[derive(Debug, Default, Eq, PartialEq)]
pub enum Vcs {
    #[default]
    Git,
}

impl Vcs {
    fn from_url(url: &url::Url) -> Self {
        let url = url.as_str();
        if url.ends_with(GIT_EXTENSION) {
            Self::Git
        } else {
            Default::default()
        }
    }

    fn extension(&self) -> &'static str {
        match self {
            Self::Git => GIT_EXTENSION,
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub enum Scheme {
    #[default]
    Https,
    Ssh,
}

impl FromStr for Scheme {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "https" => Self::Https,
            "ssh" => Self::Ssh,
            _ => Err(anyhow!("Unknown URL scheme found: {}", s))?,
        })
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub enum Host {
    #[default]
    GitHub,
    Unknown(String),
}

impl FromStr for Host {
    type Err = Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Infallible> {
        Ok(match s.to_ascii_lowercase().as_str() {
            GITHUB_COM => Self::GitHub,
            _ => Self::Unknown(s.to_string()),
        })
    }
}

impl ToString for Host {
    fn to_string(&self) -> String {
        match self {
            Self::GitHub => GITHUB_COM.to_string(),
            Self::Unknown(s) => s.clone(),
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Url {
    pub vcs: Vcs,
    pub scheme: Scheme,
    pub user: Option<String>,
    pub host: Host,
    pub owner: String,
    pub repo: String,
}

impl Url {
    pub fn from_str(s: &str, default_owner: Option<&str>) -> Result<Self> {
        match s.contains("://") {
            true => Self::from_url(&url::Url::from_str(s)?),
            _ => Self::from_pattern(s, default_owner),
        }
    }

    fn from_url(url: &url::Url) -> Result<Self> {
        let mut segments = url
            .path_segments()
            .ok_or_else(|| anyhow!("Could not parse path segments from the URL: {}", url))?;

        Ok(Self {
            vcs: Vcs::from_url(url),
            scheme: Scheme::from_str(url.scheme())?,
            user: match url.username().is_empty() {
                true => None,
                _ => Some(url.username().to_string()),
            },
            host: Host::from_str(
                url.host_str()
                    .ok_or_else(|| anyhow!("Could not find hostname from the URL: {}", url))?,
            )?,
            owner: segments
                .next()
                .ok_or_else(|| anyhow!("Could not find owner from the URL: {}", url))?
                .to_string(),
            repo: Self::remove_extensions(
                segments.next().ok_or_else(|| {
                    anyhow!("Could not find repository name from the URL: {}", url)
                })?,
            ),
        })
    }

    fn from_pattern(s: &str, default_owner: Option<&str>) -> Result<Self> {
        lazy_static! {
            static ref REPO: Regex =
                Regex::new(r"^(?P<repo>[0-9A-Za-z_\.\-]+)$").unwrap();

            static ref ORG_REPO: Regex =
                Regex::new(r"^(?P<org>[0-9A-Za-z_\.\-]+)/(?P<repo>[0-9A-Za-z_\.\-]+)$").unwrap();

            static ref HOST_ORG_REPO: Regex =
                Regex::new(r"^(?P<host>[0-9A-Za-z\.\-]+)[:/](?P<org>[0-9A-Za-z_\.\-]+)/(?P<repo>[0-9A-Za-z_\.\-]+)$").unwrap();

            static ref SSH: Regex =
                Regex::new(r"^(?P<user>[0-9A-Za-z\-]+)@(?P<host>[0-9A-Za-z\.\-]+):(?P<org>[0-9A-Za-z_\.\-]+)/(?P<repo>[0-9A-Za-z_\.\-]+)$").unwrap();
        }

        macro_rules! pattern {
            ($n: ident, $f: expr) => {
                if $n.is_match(s) {
                    let captures: Captures = $n
                        .captures(s)
                        .ok_or_else(|| anyhow!("Could not capture from the pattern"))?;

                    return $f(captures);
                }
            };
        }

        macro_rules! group {
            ($c: expr, $n: literal) => {
                $c.name($n)
                    .map(|o| o.as_str().to_string())
                    .unwrap_or_default()
            };
        }

        if let Some(owner) = default_owner {
            pattern!(REPO, |c: Captures| Ok(Self {
                owner: owner.to_string(),
                repo: Self::remove_extensions(&group!(c, "repo")),
                ..Default::default()
            }));
        }

        pattern!(ORG_REPO, |c: Captures| Ok(Self {
            owner: group!(c, "org"),
            repo: Self::remove_extensions(&group!(c, "repo")),
            ..Default::default()
        }));

        pattern!(HOST_ORG_REPO, |c: Captures| Ok(Self {
            host: Host::from_str(group!(c, "host").as_str())?,
            owner: group!(c, "org"),
            repo: Self::remove_extensions(&group!(c, "repo")),
            ..Default::default()
        }));

        pattern!(SSH, |c: Captures| Ok(Self {
            scheme: Scheme::Ssh,
            user: Some(group!(c, "user")),
            host: Host::from_str(group!(c, "host").as_str())?,
            owner: group!(c, "org"),
            repo: Self::remove_extensions(&group!(c, "repo")),
            ..Default::default()
        }));

        bail!("The input did not match any pattern: {}", s)
    }

    fn remove_extensions(s: &str) -> String {
        EXTENSIONS
            .iter()
            .fold_while(s.to_string(), |v, i| {
                let trimmed = v.trim_end_matches(i);
                match trimmed != v.as_str() {
                    true => FoldWhile::Done(trimmed.to_string()),
                    _ => FoldWhile::Continue(v),
                }
            })
            .into_inner()
    }
}

impl FromStr for Url {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Url::from_str(s, None)
    }
}

impl ToString for Url {
    fn to_string(&self) -> String {
        let authority = match &self.user {
            Some(u) => format!("{}@{}", u, self.host.to_string()),
            _ => self.host.to_string(),
        };

        match self.scheme {
            Scheme::Https => {
                format!(
                    "https://{}/{}/{}{}",
                    authority,
                    self.owner,
                    self.repo,
                    self.vcs.extension(),
                )
            }
            Scheme::Ssh => {
                format!(
                    "{}:{}/{}{}",
                    authority,
                    self.owner,
                    self.repo,
                    self.vcs.extension(),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_from_url_https() {
        let url = url::Url::parse("https://github.com/siketyan/siketyan.github.io.git").unwrap();

        assert_eq!(
            Url {
                vcs: Vcs::Git,
                scheme: Scheme::Https,
                user: None,
                host: Host::GitHub,
                owner: "siketyan".to_string(),
                repo: "siketyan.github.io".to_string()
            },
            Url::from_url(&url).unwrap(),
        )
    }

    #[test]
    fn parse_from_url_ssh() {
        let url = url::Url::parse("ssh://git@github.com/siketyan/siketyan.github.io.git").unwrap();

        assert_eq!(
            Url {
                vcs: Vcs::Git,
                scheme: Scheme::Ssh,
                user: Some("git".to_string()),
                host: Host::GitHub,
                owner: "siketyan".to_string(),
                repo: "siketyan.github.io".to_string()
            },
            Url::from_url(&url).unwrap(),
        )
    }

    #[test]
    fn parse_from_pattern_org_repo() {
        assert_eq!(
            Url {
                vcs: Vcs::Git,
                scheme: Scheme::Https,
                user: None,
                host: Host::GitHub,
                owner: "siketyan".to_string(),
                repo: "siketyan.github.io".to_string()
            },
            Url::from_pattern("siketyan/siketyan.github.io").unwrap(),
        )
    }

    #[test]
    fn parse_from_pattern_host_org_repo() {
        assert_eq!(
            Url {
                vcs: Vcs::Git,
                scheme: Scheme::Https,
                user: None,
                host: Host::Unknown("gitlab.com".to_string()),
                owner: "siketyan".to_string(),
                repo: "siketyan.github.io".to_string()
            },
            Url::from_pattern("gitlab.com:siketyan/siketyan.github.io").unwrap(),
        )
    }

    #[test]
    fn parse_from_pattern_ssh() {
        assert_eq!(
            Url {
                vcs: Vcs::Git,
                scheme: Scheme::Ssh,
                user: Some("git".to_string()),
                host: Host::GitHub,
                owner: "siketyan".to_string(),
                repo: "siketyan.github.io".to_string()
            },
            Url::from_pattern("git@github.com:siketyan/siketyan.github.io.git").unwrap(),
        )
    }

    #[test]
    fn to_string_https() {
        assert_eq!(
            "https://github.com/siketyan/siketyan.github.io.git",
            Url {
                vcs: Vcs::Git,
                scheme: Scheme::Https,
                user: None,
                host: Host::GitHub,
                owner: "siketyan".to_string(),
                repo: "siketyan.github.io".to_string()
            }
            .to_string()
            .as_str(),
        )
    }

    #[test]
    fn to_string_ssh() {
        assert_eq!(
            "git@github.com:siketyan/siketyan.github.io.git",
            Url {
                vcs: Vcs::Git,
                scheme: Scheme::Ssh,
                user: Some("git".to_string()),
                host: Host::GitHub,
                owner: "siketyan".to_string(),
                repo: "siketyan.github.io".to_string()
            }
            .to_string()
            .as_str(),
        )
    }
}
