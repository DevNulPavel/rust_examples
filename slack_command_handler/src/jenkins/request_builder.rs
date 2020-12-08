use std::{
    sync::{
        Arc
    }
};
use reqwest::{
    Client,
    RequestBuilder
};

struct RequestBuilderInternal{
    client: Client,
    jenkins_user: String,
    jenkins_api_token: String
}

pub struct JenkinsRequestBuilder{
    internal: Arc<RequestBuilderInternal>
}

impl Clone for JenkinsRequestBuilder {
    fn clone(&self) -> Self {
        JenkinsRequestBuilder{
            internal: self.internal.clone()
        }
    }
}

impl JenkinsRequestBuilder {
    pub fn new(client: Client, jenkins_user: String, jenkins_api_token: String) -> JenkinsRequestBuilder {
        let internal = Arc::new(RequestBuilderInternal{
            client,
            jenkins_user: jenkins_user,
            jenkins_api_token: jenkins_api_token
        });

        JenkinsRequestBuilder{
            internal
        }
    }

    pub fn get_jenkins_user(&self) -> &str {
        &self.internal.jenkins_user
    }

    pub fn build_get_request(&self, url: &str) -> RequestBuilder {
        let RequestBuilderInternal{
            client: client_alt_name,
            jenkins_user,
            jenkins_api_token
        } = self.internal.as_ref();

        client_alt_name
            .get(url)
            .basic_auth(jenkins_user, Some(jenkins_api_token))
    }

    pub fn build_post_request(&self, url: &str) -> RequestBuilder {
        let RequestBuilderInternal{
            client,
            jenkins_user,
            jenkins_api_token
        } = self.internal.as_ref();

        client
            .post(url)
            .basic_auth(jenkins_user, Some(jenkins_api_token))
    }
}