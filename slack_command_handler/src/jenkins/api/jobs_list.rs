use serde::{
    Deserialize
};
use serde_json::{
    Value
};
use log::{
    debug,
    // info,
    // error
};
use crate::{
    jenkins::{
        auth::{
            JenkinsAuth
        }
    }
};


// https://jenkins.17btest.com/api?pretty=true
// https://jenkins.17btest.com/job/bbd-gplay-prod/api/json?pretty=true
// https://www.jenkins.io/doc/book/using/remote-access-api/

#[derive(Deserialize, Debug)]
pub struct JenkinsJob{
    pub name: String,
    pub url: String
}

#[derive(Deserialize, Debug)]
struct JenkinsJobsResponse{
    jobs: Vec<JenkinsJob>
}

/// Запрашиваем список возможных
pub async fn request_jenkins_jobs_list(client: &reqwest::Client, auth: &JenkinsAuth) -> Result<Vec<JenkinsJob>, reqwest::Error> {
    // https://jenkins.17btest.com/api/json?pretty=true

    // debug!("{} {}", jenkins_user, jenkins_token);

    let response = client
        .get("https://jenkins.17btest.com/api/json")
        .basic_auth(&auth.jenkins_user, Some(&auth.jenkins_api_token))
        .send()
        .await?;

    let result = response
        .json::<JenkinsJobsResponse>()
        .await?;
    
    Ok(result.jobs)
}
