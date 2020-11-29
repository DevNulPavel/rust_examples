use serde::{
    Deserialize
};
use actix_web::{
    web::{
        Bytes
    }
};
use log::{
    debug
};
use super::{
    error::{
        JenkinsError
    },
    job_parameter::{
        Parameter
    }
};


#[derive(Deserialize, Debug)]
pub struct JenkinsJobInfo{
    pub name: String,
    pub url: String
}

pub struct JenkinsJob{
    jenkins_user: String,
    jenkins_api_token: String,
    info: JenkinsJobInfo
}

impl<'a> JenkinsJob {
    pub fn new(jenkins_user: &str, jenkins_api_token: &str, info: JenkinsJobInfo) -> JenkinsJob {
        JenkinsJob{
            jenkins_user: jenkins_user.to_owned(),
            jenkins_api_token: jenkins_api_token.to_owned(),
            info
        }
    }

    pub fn get_info(&self) -> &JenkinsJobInfo {
        return &self.info;
    }

    /// Запрашиваем список возможных
    pub async fn request_jenkins_job_info(&self) -> Result<Vec<Parameter>, JenkinsError> {
        // https://jenkins.17btest.com/job/bbd-gplay-prod/config.xml
        // debug!("{} {}", jenkins_user, jenkins_token);

        // Примеры
        // https://github.com/RReverser/serde-xml-rs/blob/master/tests/test.rs
        #[derive(Debug, Deserialize)]
        struct Values {
            #[serde(rename = "$value", default)]
            values: Vec<Parameter>,
        }
        #[derive(Deserialize, Debug)]
        struct InfoParameters{
            #[serde(rename = "parameterDefinitions")]
            parameters: Values
        }
        #[derive(Deserialize, Debug)]
        struct InfoProperty{
            #[serde(rename = "hudson.model.ParametersDefinitionProperty")]
            parameters: InfoParameters
        }
        #[derive(Deserialize, Debug)]
        struct InfoResponse{
            properties: InfoProperty
        }

        let result: InfoResponse = {
            let mut response = {
                let client = ClientBuilder::new()
                    .basic_auth(&self.jenkins_user, Some(&self.jenkins_api_token))
                    .finish();

                let job_info_url = format!("https://jenkins.17btest.com/job/{}/config.xml", self.info.name);
                client
                    .get(job_info_url.as_str())
                    .send()
                    .await?
            };

            let xml_data: Bytes = response
                .body()
                .await?;
            
            let text = std::str::from_utf8(xml_data.as_ref())?;

            serde_xml_rs::from_str(text)?
        };

        let parameters: Vec<Parameter> = result
            .properties
            .parameters
            .parameters
            .values;

        debug!("Job info result: {:?}", parameters);
        
        Ok(parameters)
    }
}