use crate::{
    jenkins::{
        JenkinsClient
    },
    slack::{
        SlackClient,
    }
};

pub struct ApplicationData{
    pub slack_client: SlackClient,
    pub jenkins_client: JenkinsClient,
}

impl ApplicationData{
    pub fn new(slack_client: SlackClient, 
               jenkins_client: JenkinsClient) -> ApplicationData {
                   
        ApplicationData{
            slack_client,
            jenkins_client
        }
    }
}