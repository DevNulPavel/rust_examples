use crate::{
    jenkins::{
        JenkinsClient
    },
    slack::{
        SlackClient
    }
};

//#[derive(Clone)]
pub struct ApplicationData{
    pub slack_client: SlackClient,
    pub jenkins_client: JenkinsClient
}
