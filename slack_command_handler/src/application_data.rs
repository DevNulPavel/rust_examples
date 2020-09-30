
#[derive(Clone)]
pub struct ApplicationData{
    pub slack_api_token: String,
    pub jenkins_user: String,
    pub jenkins_api_token: String,
    pub http_client: reqwest::Client
}
