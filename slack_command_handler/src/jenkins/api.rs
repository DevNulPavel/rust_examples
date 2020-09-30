use serde::{
    Deserialize
};
use log::{
    debug,
    // info,
    // error
};


// https://jenkins.17btest.com/api/
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

pub async fn request_jenkins_jobs_list(client: &reqwest::Client, jenkins_user: &str, jenkins_token: &str) -> Result<Vec<JenkinsJob>, reqwest::Error> {
    // https://jenkins.17btest.com/api/json?pretty=true

    debug!("{} {}", jenkins_user, jenkins_token);

    let response = client
        .post("https://jenkins.17btest.com/api/json")
        .basic_auth(jenkins_user, Some(jenkins_token))
        .send()
        .await?;

    // let text = response
    //     .text()
    //     .await?;

    // debug!("Jobs response: {}", text.as_str());

    // let resp = serde_json::from_str::<JenkinsJobsResponse>(text.as_str()).unwrap();
    // Ok(resp.jobs)

    let result = response
        .json::<JenkinsJobsResponse>()
        .await?;
    
    Ok(result.jobs)
}