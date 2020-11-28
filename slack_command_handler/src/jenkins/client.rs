use actix_web::{
    client::{
        Client,
        ClientBuilder
    }
};
// use log::{
//     error,
//     debug
// };
use serde::{
    Deserialize
};
use super::{
    error::{
        JenkinsError
    },
    job::{
        JenkinsJobInfo,
        JenkinsJob
    }
};


pub struct JenkinsClient{
    client: Client
}

impl JenkinsClient {
    pub fn new(jenkins_user: &str, jenkins_api_token: &str) -> JenkinsClient {
        let client = ClientBuilder::new()
            .basic_auth(jenkins_user, Some(jenkins_api_token))
            .finish();

        JenkinsClient{
            client
        }
    }

    /// Запрашиваем список возможных таргетов
    pub async fn request_jenkins_jobs_list<'a>(&'a self) -> Result<Vec<JenkinsJob<'a>>, JenkinsError> {
        // https://jenkins.17btest.com/api?pretty=true
        // https://jenkins.17btest.com/job/bbd-gplay-prod/api/json?pretty=true
        // https://www.jenkins.io/doc/book/using/remote-access-api/
        // https://jenkins.17btest.com/api/json?pretty=true

        #[derive(Deserialize, Debug)]
        struct JenkinsJobsResponse{
            jobs: Vec<JenkinsJobInfo>
        }

        let response = self.client
            .get("https://jenkins.17btest.com/api/json")
            .send()
            .await?
            .json::<JenkinsJobsResponse>()
            .await?;

        let result: Vec<JenkinsJob<'a>> = response
            .jobs
            .into_iter()
            .map(|info|{
                JenkinsJob::new(&self.client, info)
            })
            .collect();

        Ok(result)
    }

}