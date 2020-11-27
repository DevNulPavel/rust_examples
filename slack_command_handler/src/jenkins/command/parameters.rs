use std::{
    fmt
};
use serde::{
    Serialize,
    Deserialize
};

#[derive(Deserialize, Serialize)]
pub struct SlackCommandParameters{
    pub user_id: String,
    pub user_name: String,
    pub trigger_id: String,
    pub command: String,

    // pub token: String,
    // pub text: String,
    // channel_id: String,
    // team_id: String,
    // team_domain: String,
    // enterprise_id: String,
    // enterprise_name: String,
    // channel_name: String,
    // response_url: String,
    // api_app_id: String

    // Так можно получить прочие необязательные параметры
    // https://serde.rs/attr-flatten.html
    // #[serde(flatten)]
    // extra: HashMap<String, Value>
}

impl fmt::Debug for SlackCommandParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = serde_json::to_string_pretty(self).unwrap();
        f.write_str(text.as_str())
    }
}