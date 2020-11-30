use reqwest::{
    Client
};
use log::{
    // error,
    debug,
    // info
};
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
    client: Client,
    jenkins_user: String,
    jenkins_api_token: String
}

impl JenkinsClient {
    pub fn new(client: Client, jenkins_user: &str, jenkins_api_token: &str) -> JenkinsClient {
        JenkinsClient{
            client,
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

        debug!("Jenkins client params: {}, {}", self.jenkins_user, self.jenkins_api_token);

        /*let client = ClientBuilder::new()
            .basic_auth(&self.jenkins_user, Some(&self.jenkins_api_token))
            .finish();

        let mut response = client
            .get("https://jenkins.17btest.com/api/json")
            .send()
            .await?;*/
        
        /*let body: actix_web::web::Bytes = response
            .body()
            .await
            .unwrap(); // TODO: ??

        let body = std::str::from_utf8(&body).unwrap();

        debug!("Jenkins response: {}", body);

        let parsed_response = serde_json::from_str::<JenkinsJobsResponse>(body).unwrap();*/

        let parsed_response = self.client
            .get("https://jenkins.17btest.com/api/json")
            .basic_auth(&self.jenkins_user, Some(&self.jenkins_api_token))
            .send()
            .await
            .map_err(|err|{
                JenkinsError::BodyParseError(err, "Jobs request body parse error".to_owned())
            })?
            .json::<JenkinsJobsResponse>()
            .await
            .map_err(|err|{
                JenkinsError::JsonParseError(err, "Jobs request JSON parse error".to_owned())
            })?;

        let result: Vec<JenkinsJob> = parsed_response
            .jobs
            .into_iter()
            .map(|info|{
                JenkinsJob::new(self.client.clone(), &self.jenkins_user, &self.jenkins_api_token, info)
            })
            .collect();

        Ok(result)
    }

}
