use crate::url::Url;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ProfileRef {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub profile: ProfileRef,
    pub host: Option<String>,
    pub owner: Option<String>,
    pub repo: Option<String>,
}

impl Rule {
    pub fn matches(&self, url: &Url) -> bool {
        self.host
            .as_deref()
            .map(|h| h == url.host.to_string())
            .unwrap_or(true)
            && self
                .owner
                .as_deref()
                .map(|o| o == url.owner)
                .unwrap_or(true)
            && self.repo.as_deref().map(|r| r == url.repo).unwrap_or(true)
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct Rules(Vec<Rule>);

impl Rules {
    pub fn resolve(&self, url: &Url) -> Option<&Rule> {
        self.0.iter().find(|rule| rule.matches(url))
    }
}
