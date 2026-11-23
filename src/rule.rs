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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::url::Patterns;

    fn url(s: &str) -> Url {
        Url::from_str(s, &Patterns::default(), None).unwrap()
    }

    fn rule(host: Option<&str>, owner: Option<&str>, repo: Option<&str>) -> Rule {
        Rule {
            profile: ProfileRef {
                name: "default".to_string(),
            },
            host: host.map(String::from),
            owner: owner.map(String::from),
            repo: repo.map(String::from),
        }
    }

    #[test]
    fn empty_rule_matches_everything() {
        let r = rule(None, None, None);

        assert!(r.matches(&url("siketyan/ghr")));
        assert!(r.matches(&url("example.com/foo/bar")));
    }

    #[test]
    fn matches_by_host() {
        let r = rule(Some("github.com"), None, None);

        assert!(r.matches(&url("siketyan/ghr")));
        assert!(!r.matches(&url("example.com/siketyan/ghr")));
    }

    #[test]
    fn matches_by_owner() {
        let r = rule(None, Some("siketyan"), None);

        assert!(r.matches(&url("siketyan/ghr")));
        assert!(r.matches(&url("example.com/siketyan/other")));
        assert!(!r.matches(&url("someone/ghr")));
    }

    #[test]
    fn matches_by_repo() {
        let r = rule(None, None, Some("ghr"));

        assert!(r.matches(&url("siketyan/ghr")));
        assert!(!r.matches(&url("siketyan/other")));
    }

    #[test]
    fn requires_all_conditions_to_match() {
        let r = rule(Some("github.com"), Some("siketyan"), Some("ghr"));

        assert!(r.matches(&url("siketyan/ghr")));
        assert!(!r.matches(&url("example.com/siketyan/ghr")));
        assert!(!r.matches(&url("someone/ghr")));
        assert!(!r.matches(&url("siketyan/other")));
    }

    #[test]
    fn resolve_returns_first_matching_rule() {
        let rules = toml::from_str::<Wrapper>(
            r#"
            [[rules]]
            profile.name = "work"
            owner = "acme"

            [[rules]]
            profile.name = "personal"
            "#,
        )
        .unwrap()
        .rules;

        assert_eq!(
            "work",
            rules.resolve(&url("acme/api")).unwrap().profile.name,
        );
        assert_eq!(
            "personal",
            rules.resolve(&url("siketyan/ghr")).unwrap().profile.name,
        );
    }

    #[test]
    fn resolve_returns_none_without_rules() {
        assert!(Rules::default().resolve(&url("siketyan/ghr")).is_none());
    }

    #[derive(Deserialize)]
    struct Wrapper {
        rules: Rules,
    }
}
