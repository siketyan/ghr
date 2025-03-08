use std::collections::HashMap;
use std::fmt::Formatter;
use std::ops::Deref;
use std::result::Result as StdResult;

use anyhow::Result;
use git2::Repository;
use itertools::Itertools;
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use toml::Table;
use toml::value::Value;

use crate::git::exclude::{File, Node};
use crate::rule::ProfileRef;

#[derive(Debug, Default)]
pub struct Configs {
    map: HashMap<String, String>,
}

impl Configs {
    fn to_toml(&self) -> Table {
        let mut map = Table::new();
        for (key, value) in &self.map {
            let segments: Vec<_> = key.split('.').collect();
            let mut inner = &mut map;

            for segment in segments.iter().take(segments.len() - 1) {
                if !inner.contains_key(*segment) {
                    inner.insert(segment.to_string(), Value::Table(Table::new()));
                }

                inner = match inner.get_mut(*segment).unwrap() {
                    Value::Table(table) => table,
                    _ => panic!("unexpected non-table value"),
                }
            }

            inner.insert(
                segments.last().unwrap().to_string(),
                Value::String(value.to_string()),
            );
        }

        map
    }

    fn extend_from_toml(&mut self, input: &Value, current_key: &str) {
        match input {
            Value::String(value) => {
                self.map.insert(current_key.to_string(), value.clone());
            }
            Value::Table(table) => {
                for (key, value) in table {
                    self.extend_from_toml(
                        value,
                        &match key.is_empty() {
                            true => key.clone(),
                            _ => format!("{}.{}", current_key, key),
                        },
                    );
                }
            }
            _ => (),
        }
    }
}

impl Deref for Configs {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<'de> Deserialize<'de> for Configs {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ConfigsVisitor)
    }
}

impl Serialize for Configs {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        self.to_toml()
            .iter()
            .map(|(k, v)| map.serialize_entry(k, v))
            .try_collect::<_, (), _>()?;

        map.end()
    }
}

struct ConfigsVisitor;

impl<'de> Visitor<'de> for ConfigsVisitor {
    type Value = Configs;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("git configs of a profile")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut configs = Configs::default();

        while let Some((key, value)) = map.next_entry::<String, Value>()? {
            configs.extend_from_toml(&value, &key)
        }

        Ok(configs)
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Profile {
    #[serde(default)]
    pub excludes: Vec<String>,
    #[serde(default, flatten)]
    pub configs: Configs,
}

impl Profile {
    pub fn apply(&self, repo: &Repository) -> Result<()> {
        let path = repo.workdir().unwrap();
        let mut exclude = File::load(path)?;
        for value in &self.excludes {
            exclude.add_or_noop(Node::Exclude(value.to_string()));
        }

        exclude.save(path)?;

        let mut config = repo.config()?;
        for (key, value) in &self.configs.map {
            config.set_str(key, value)?;
        }

        Ok(())
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct Profiles {
    #[serde(flatten)]
    map: HashMap<String, Profile>,
}

impl Profiles {
    pub fn resolve(&self, r: &ProfileRef) -> Option<(&str, &Profile)> {
        self.get_key_value(&r.name).map(|(s, p)| (s.as_str(), p))
    }
}

impl Deref for Profiles {
    type Target = HashMap<String, Profile>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_git_configs() {
        let toml = r#"
        user.name = "User Taro"
        user.email = "taro@example.com"
        user.signingkey = "ABCDEFGHIJKLMNOP"
        "#;

        let profile = toml::from_str::<Profile>(toml).unwrap();
        let configs = &profile.configs;

        assert_eq!("User Taro", configs.get("user.name").unwrap().as_str());
        assert_eq!(
            "taro@example.com",
            configs.get("user.email").unwrap().as_str(),
        );
        assert_eq!(
            "ABCDEFGHIJKLMNOP",
            configs.get("user.signingkey").unwrap().as_str(),
        );
    }
}
