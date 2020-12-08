// use reqwest::{
//     Client
// };
// use log::{
    // error,
    //debug,
    // info
// };
use serde::{
    Deserialize
};
use super::{
    request_builder::{
        JenkinsRequestBuilder
    },
    error::{
        JenkinsError
    },
    target::{
        JenkinsTargetInfo,
        JenkinsTarget
    }
};

pub struct JenkinsClient{
    client: JenkinsRequestBuilder
}

impl JenkinsClient {
    pub fn new(client: JenkinsRequestBuilder) -> JenkinsClient {
        //debug!("Jenkins client params: {}, {}", jenkins_user, jenkins_api_token);

        JenkinsClient{
            client
        }
    }

    pub fn get_jenkins_user(&self) -> &str {
        self.client.get_jenkins_user()
    }

    /// Запрашиваем список возможных таргетов
    pub async fn request_jenkins_targets_list<'a>(&'a self) -> Result<Vec<JenkinsTarget>, JenkinsError> {
        // https://jenkins.17btest.com/api?pretty=true
        // https://jenkins.17btest.com/job/bbd-gplay-prod/api/json?pretty=true
        // https://www.jenkins.io/doc/book/using/remote-access-api/
        // https://jenkins.17btest.com/api/json?pretty=true

        #[derive(Deserialize, Debug)]
        struct JenkinsTargetsResponse{
            jobs: Vec<JenkinsTargetInfo>
        }

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

        let parsed_response = serde_json::from_str::<JenkinsTargetsResponse>(body).unwrap();*/

        let parsed_response = self
            .client
            .build_get_request("https://jenkins.17btest.com/api/json")
            .send()
            .await
            .map_err(|err|{
                JenkinsError::BodyParseError(err, "Jobs request body parse error".to_owned())
            })?
            .json::<JenkinsTargetsResponse>()
            .await
            .map_err(|err|{
                JenkinsError::JsonParseError(err, "Jobs request JSON parse error".to_owned())
            })?;

        let result: Vec<JenkinsTarget> = parsed_response
            .jobs
            .into_iter()
            .map(|info|{
                JenkinsTarget::new(self.client.clone(), info)
            })
            .collect();

        Ok(result)
    }

}
