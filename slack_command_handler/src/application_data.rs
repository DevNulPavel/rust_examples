use crate::{
    jenkins::{
        JenkinsAuth
    },
    slack::{
        SlackClient
    }
};

//#[derive(Clone)]
pub struct ApplicationData{
    //pub slack_api_token: String,
    pub jenkins_auth: JenkinsAuth,
    pub http_client: reqwest::Client,
    pub slack_client: SlackClient
}
