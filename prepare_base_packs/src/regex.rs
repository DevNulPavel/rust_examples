use fancy_regex::Regex;
use serde::{Deserialize, Deserializer};
use std::ops::Deref;

#[derive(Debug)]
pub struct Re(pub Regex);

impl<'de> Deserialize<'de> for Re {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let regex = Regex::new(&s).map_err(serde::de::Error::custom)?;
        Ok(Re(regex))
    }
}
impl Into<Regex> for Re {
    fn into(self) -> Regex {
        self.0
    }
}
impl Deref for Re {
    type Target = Regex;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<Regex> for Re {
    fn as_ref(&self) -> &Regex {
        &self.0
    }
}