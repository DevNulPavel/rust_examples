use serde::{
    Deserialize
};

#[derive(Deserialize, Debug)]
pub struct SlackCommandParameters{
    pub token: String,
    pub user_id: String,
    pub user_name: String,
    pub trigger_id: String,
    pub command: String,
    pub text: String,
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