use regex::Regex;
use serde::Deserialize;
use serde_regex;

#[derive(Debug, Deserialize)]
pub struct FilterConfig {
    #[serde(with = "serde_regex")]
    pub allowed_keys_regex: Vec<Regex>,
}
