use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::ops::Deref;

use crate::rule::ProfileRef;
use anyhow::Result;
use git2::Config;
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use toml::value::Value;
use toml::Table;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Profile {
    #[serde(default, flatten)]
    pub configs: Configs,
}

#[derive(Debug, Default)]
pub struct Configs(pub HashMap<String, String>);

impl<'de> Deserialize<'de> for Configs {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
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

        for (key, value) in expand_value(&self.0) {
            map.serialize_entry(&key, &value)?;
        }

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
            flatten_value(&value, &key, &mut configs.0)
        }

        Ok(configs)
    }
}

fn flatten_value(input: &Value, current_key: &str, output: &mut HashMap<String, String>) {
    match input {
        Value::String(value) => {
            output.insert(current_key.to_string(), value.clone());
        }
        Value::Table(table) => {
            for (key, value) in table {
                let new_key = if current_key.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", current_key, key)
                };
                flatten_value(value, &new_key, output)
            }
        }
        _ => (),
    }
}

fn expand_value(input: &HashMap<String, String>) -> Table {
    let mut map = Table::new();
    for (key, value) in input {
        let segments: Vec<_> = key.split('.').collect();
        let mut inner = map.borrow_mut();

        for segment in segments.as_slice()[0..segments.len() - 1].iter() {
            let segment = segment.to_string();
            if !inner.contains_key(&segment) {
                inner.insert(segment.clone(), Value::Table(Table::new()));
            }
            inner = match inner.get_mut(&segment).unwrap() {
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

impl Profile {
    pub fn apply(&self, config: &mut Config) -> Result<()> {
        for (key, value) in &self.configs.0 {
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
