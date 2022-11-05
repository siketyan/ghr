use std::collections::HashMap;
use std::ops::Deref;

use crate::rule::ProfileRef;
use anyhow::Result;
use git2::Config;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct User {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Profile {
    #[serde(default)]
    pub user: Option<User>,
}

impl Profile {
    pub fn apply(&self, config: &mut Config) -> Result<()> {
        if let Some(name) = self.user.as_ref().and_then(|u| u.name.as_deref()) {
            config.set_str("user.name", name)?;
        }

        if let Some(email) = self.user.as_ref().and_then(|u| u.email.as_deref()) {
            config.set_str("user.email", email)?;
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
