use actix_web::{
    client::{
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
    jenkins_user: String,
    jenkins_api_token: String
}

impl JenkinsClient {
    pub fn new(jenkins_user: &str, jenkins_api_token: &str) -> JenkinsClient {
        JenkinsClient{
            jenkins_user: jenkins_user.to_owned(),
            jenkins_api_token: jenkins_api_token.to_owned()
        }
    }

    /// Запрашиваем список возможных таргетов
    pub async fn request_jenkins_jobs_list<'a>(&'a self) -> Result<Vec<JenkinsJob>, JenkinsError> {
        // https://jenkins.17btest.com/api?pretty=true
        // https://jenkins.17btest.com/job/bbd-gplay-prod/api/json?pretty=true
        // https://www.jenkins.io/doc/book/using/remote-access-api/
        // https://jenkins.17btest.com/api/json?pretty=true

        #[derive(Deserialize, Debug)]
        struct JenkinsJobsResponse{
            jobs: Vec<JenkinsJobInfo>
        }

        let client = ClientBuilder::new()
            .basic_auth(&self.jenkins_user, Some(&self.jenkins_api_token))
            .finish();

        let response = client
            .get("https://jenkins.17btest.com/api/json")
            .send()
            .await?
            .json::<JenkinsJobsResponse>()
            .await?;

        let result: Vec<JenkinsJob> = response
            .jobs
            .into_iter()
            .map(|info|{
                JenkinsJob::new(&self.jenkins_user, &self.jenkins_api_token, info)
            })
            .collect();

        Ok(result)
    }

}