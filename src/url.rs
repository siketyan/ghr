use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use anyhow::{anyhow, bail, Error, Result};
use itertools::FoldWhile;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use serde_with::DeserializeFromStr;

const GITHUB_COM: &str = "github.com";

const GIT_EXTENSION: &str = ".git";
const EXTENSIONS: &[&str] = &[GIT_EXTENSION];

lazy_static! {
    static ref SSH: Pattern = Pattern::from(
        Regex::new(r"^(?P<user>[0-9A-Za-z\-]+)@(?P<host>[0-9A-Za-z\.\-]+):(?P<owner>[0-9A-Za-z_\.\-]+)/(?P<repo>[0-9A-Za-z_\.\-]+)$")
            .unwrap(),
    )
        .with_scheme(Scheme::Ssh)
        .with_infer();

    static ref HOST_ORG_REPO: Pattern = Pattern::from(
        Regex::new(r"^(?P<host>[0-9A-Za-z\.\-]+)[:/](?P<owner>[0-9A-Za-z_\.\-]+)/(?P<repo>[0-9A-Za-z_\.\-]+)$")
            .unwrap(),
    )
        .with_infer();

    static ref ORG_REPO: Pattern = Pattern::from(
        Regex::new(r"^(?P<owner>[0-9A-Za-z_\.\-]+)/(?P<repo>[0-9A-Za-z_\.\-]+)$")
            .unwrap(),
    )
        .with_infer();

    static ref REPO: Pattern = Pattern::from(
        Regex::new(r"^(?P<repo>[0-9A-Za-z_\.\-]+)$")
            .unwrap(),
    )
        .with_infer();
}

#[derive(Debug)]
pub struct Match {
    pub vcs: Option<Vcs>,
    pub scheme: Option<Scheme>,
    pub user: Option<String>,
    pub host: Option<Host>,
    pub owner: Option<String>,
    pub repo: String,
    pub raw: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pattern {
    #[serde(with = "serde_regex")]
    regex: Regex,
    vcs: Option<Vcs>,
    scheme: Option<Scheme>,
    user: Option<String>,
    host: Option<Host>,
    owner: Option<String>,
    url: Option<String>,
    infer: Option<bool>,
}

impl Pattern {
    #[inline]
    pub fn with_scheme(mut self, s: Scheme) -> Self {
        self.scheme = Some(s);
        self
    }

    #[inline]
    pub fn with_infer(mut self) -> Self {
        self.infer = Some(true);
        self
    }

    pub fn matches(&self, s: &str) -> Option<Match> {
        self.regex.captures(s).and_then(|c| {
            let repo = match c.name("repo") {
                Some(v) => v.as_str().to_string(),
                _ => return None,
            };

            let mut m = Match {
                vcs: c
                    .name("vcs")
                    .and_then(|v| Vcs::from_str(v.as_str()).ok())
                    .or(self.vcs),
                scheme: c
                    .name("scheme")
                    .and_then(|v| Scheme::from_str(v.as_str()).ok())
                    .or(self.scheme),
                user: c
                    .name("user")
                    .map(|v| v.as_str().to_string())
                    .or(self.user.clone()),
                host: c
                    .name("host")
                    .and_then(|v| Host::from_str(v.as_str()).ok())
                    .or(self.host.clone()),
                owner: c
                    .name("owner")
                    .map(|v| v.as_str().to_string())
                    .or(self.owner.clone()),
                repo,
                raw: None,
            };

            m.raw = match self.infer.unwrap_or_default() {
                true => None,
                _ => self
                    .url
                    .as_ref()
                    .map(|u| {
                        u.replace("{{vcs}}", &m.vcs.map(|v| v.to_string()).unwrap_or_default())
                            .replace(
                                "{{scheme}}",
                                &m.scheme.map(|s| s.to_string()).unwrap_or_default(),
                            )
                            .replace("{{user}}", &m.user.clone().unwrap_or_default())
                            .replace(
                                "{{host}}",
                                &m.host.clone().map(|h| h.to_string()).unwrap_or_default(),
                            )
                            .replace("{{owner}}", &m.owner.clone().unwrap_or_default())
                            .replace("{{repo}}", &m.repo)
                    })
                    .or(Some(s.to_string())),
            };

            Some(m)
        })
    }
}

impl From<Regex> for Pattern {
    fn from(value: Regex) -> Self {
        Self {
            regex: value,
            vcs: None,
            scheme: None,
            user: None,
            host: None,
            owner: None,
            url: None,
            infer: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Patterns(Vec<Pattern>);

impl Patterns {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn add(&mut self, p: Pattern) {
        self.0.push(p);
    }

    #[inline]
    pub fn with(mut self, p: Pattern) -> Self {
        self.add(p);
        self
    }

    pub fn with_defaults(self) -> Self {
        self.with(SSH.clone())
            .with(HOST_ORG_REPO.clone())
            .with(ORG_REPO.clone())
            .with(REPO.clone())
    }

    pub fn matches(&self, s: &str) -> Option<Match> {
        self.0.iter().find_map(|p| p.matches(s))
    }
}

impl Default for Patterns {
    fn default() -> Self {
        Self::new().with_defaults()
    }
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, DeserializeFromStr)]
pub enum Vcs {
    #[default]
    Git,
}

impl Vcs {
    fn from_url(url: &url::Url) -> Option<Self> {
        let url = url.as_str();
        if url.ends_with(GIT_EXTENSION) {
            Some(Self::Git)
        } else {
            None
        }
    }

    fn extension(&self) -> &'static str {
        match self {
            Self::Git => GIT_EXTENSION,
        }
    }
}

impl FromStr for Vcs {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "git" => Self::Git,
            _ => Err(anyhow!("Unknown VCS found: {}", s))?,
        })
    }
}

impl Display for Vcs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Git => write!(f, "git"),
        }
    }
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, DeserializeFromStr)]
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

impl Display for Scheme {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Https => write!(f, "https"),
            Self::Ssh => write!(f, "ssh"),
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, DeserializeFromStr)]
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

impl Display for Host {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitHub => write!(f, "{}", GITHUB_COM),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct PartialUrl {
    pub vcs: Option<Vcs>,
    pub scheme: Option<Scheme>,
    pub user: Option<String>,
    pub host: Option<Host>,
    pub owner: Option<String>,
    pub repo: String,
    pub raw: Option<String>,
}

impl PartialUrl {
    pub fn from_str(s: &str, p: &Patterns) -> Result<Self> {
        Self::from_pattern(s, p).or_else(|e| match s.contains("://") {
            true => Self::from_url(&url::Url::from_str(s)?),
            _ => Err(e),
        })
    }

    fn from_url(url: &url::Url) -> Result<Self> {
        let mut segments = url
            .path_segments()
            .ok_or_else(|| anyhow!("Could not parse path segments from the URL: {}", url))?;

        let scheme = Scheme::from_str(url.scheme())?;

        Ok(Self {
            vcs: Vcs::from_url(url),
            scheme: Some(scheme),
            user: match url.username().is_empty() {
                true => None,
                _ => Some(url.username().to_string()),
            },
            host: match url.host_str() {
                Some(h) => Some(Host::from_str(h)?),
                _ => None,
            },
            owner: segments.next().map(|s| s.to_string()),
            repo: Self::remove_extensions(
                segments.next().ok_or_else(|| {
                    anyhow!("Could not find repository name from the URL: {}", url)
                })?,
            ),
            raw: match scheme {
                // HTTPS URLs can be used directly on cloning, so we prefer it than inferred one.
                // SSH URLs are not; Git only accepts 'git@github.com:org/repo.git' style.
                Scheme::Https => Some(url.to_string()),
                _ => None,
            },
        })
    }

    fn from_match(m: Match) -> Option<Self> {
        Some(Self {
            vcs: m.vcs,
            scheme: m.scheme,
            user: m.user,
            host: m.host,
            owner: m.owner,
            repo: Self::remove_extensions(&m.repo),
            raw: m.raw,
        })
    }

    fn from_pattern(s: &str, p: &Patterns) -> Result<Self> {
        p.matches(s)
            .and_then(|m| Self::from_match(m))
            .ok_or(anyhow!("The input did not match any pattern: {}", s))
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

#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct Url {
    pub vcs: Vcs,
    pub scheme: Scheme,
    pub user: Option<String>,
    pub host: Host,
    pub owner: String,
    pub repo: String,
    pub raw: Option<String>,
}

impl Url {
    pub fn from_partial(p: &PartialUrl, default_owner: Option<&str>) -> Result<Self> {
        Ok(Self {
            vcs: p.vcs.unwrap_or_default(),
            scheme: p.scheme.unwrap_or_default(),
            user: p.user.clone(),
            host: p.host.clone().unwrap_or_default(),
            owner: match &p.owner {
                Some(o) => o.to_string(),
                _ => match default_owner {
                    Some(d) => d.to_string(),
                    _ => bail!("Repository owner is not specified in the URL or pattern, and the default owner is not configured.")
                }
            },
            repo: p.repo.clone(),
            raw: p.raw.clone(),
        })
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(r) = &self.raw {
            return write!(f, "{}", r);
        }

        let authority = match &self.user {
            Some(u) => format!("{}@{}", u, &self.host),
            _ => self.host.to_string(),
        };

        match self.scheme {
            Scheme::Https => {
                write!(
                    f,
                    "https://{}/{}/{}{}",
                    authority,
                    self.owner,
                    self.repo,
                    self.vcs.extension(),
                )
            }
            Scheme::Ssh => {
                write!(
                    f,
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
                repo: "siketyan.github.io".to_string(),
                raw: Some("https://github.com/siketyan/siketyan.github.io.git".to_string()),
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
                repo: "siketyan.github.io".to_string(),
                ..Default::default()
            },
            Url::from_url(&url).unwrap(),
        )
    }

    #[test]
    fn parse_from_pattern_repo() {
        assert_eq!(
            Url {
                vcs: Vcs::Git,
                scheme: Scheme::Https,
                user: None,
                host: Host::GitHub,
                owner: "siketyan".to_string(),
                repo: "siketyan.github.io".to_string(),
                ..Default::default()
            },
            Url::from_pattern("siketyan.github.io", &Patterns::default(), Some("siketyan"))
                .unwrap(),
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
                repo: "siketyan.github.io".to_string(),
                ..Default::default()
            },
            Url::from_pattern("siketyan/siketyan.github.io", &Patterns::default(), None).unwrap(),
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
                repo: "siketyan.github.io".to_string(),
                ..Default::default()
            },
            Url::from_pattern(
                "gitlab.com:siketyan/siketyan.github.io",
                &Patterns::default(),
                None
            )
            .unwrap(),
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
                repo: "siketyan.github.io".to_string(),
                ..Default::default()
            },
            Url::from_pattern(
                "git@github.com:siketyan/siketyan.github.io.git",
                &Patterns::default(),
                None
            )
            .unwrap(),
        )
    }

    #[test]
    fn parse_from_custom_pattern() {
        let patterns = Patterns::default().with(
            Pattern::from(
                Regex::new(r"^(?P<scheme>https)://(?P<host>git\.kernel\.org)/pub/scm/linux/kernel/git/(?P<owner>.+)/(?P<repo>.+)\.git").unwrap()
            )
                .with_scheme(Scheme::Https)
        );

        assert_eq!(
            Url {
                vcs: Vcs::Git,
                scheme: Scheme::Https,
                host: Host::Unknown("git.kernel.org".to_string()),
                owner: "torvalds".to_string(),
                repo: "linux".to_string(),
                raw: Some(
                    "https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git"
                        .to_string(),
                ),
                ..Default::default()
            },
            Url::from_pattern(
                "https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git",
                &patterns,
                None
            )
            .unwrap(),
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
                repo: "siketyan.github.io".to_string(),
                ..Default::default()
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
                repo: "siketyan.github.io".to_string(),
                ..Default::default()
            }
            .to_string()
            .as_str(),
        )
    }
}
