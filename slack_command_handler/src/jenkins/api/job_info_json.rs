/*use serde::{
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

#[derive(Deserialize, Debug)]
pub struct JenkinsJobParameterDefaultBoolValue{
    pub name: String,
    pub value: bool
}

#[derive(Deserialize, Debug)]
pub struct JenkinsJobParameterDefaultStringValue{
    pub name: String,
    pub value: String
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum JenkinsJobParameter{
    #[serde(rename = "BooleanParameterDefinition")]
    Boolean{
        name: String,
        description: String,
    
        #[serde(rename = "defaultParameterValue")]
        default_value: JenkinsJobParameterDefaultBoolValue
    },
    #[serde(rename = "StringParameterDefinition")]
    String{
        name: String,
        description: String,
    
        #[serde(rename = "defaultParameterValue")]
        default_value: JenkinsJobParameterDefaultStringValue
    },
    #[serde(rename = "ExtensibleChoiceParameterDefinition")]
    Choice{
        name: String,
        description: String,
    
        #[serde(rename = "defaultParameterValue")]
        default_value: JenkinsJobParameterDefaultStringValue
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "_class")]
enum JenkinsJobInfoProperty{
    #[serde(rename = "hudson.model.ParametersDefinitionProperty")]
    Parameter{
        #[serde(rename = "parameterDefinitions")]
        parameters: Vec<JenkinsJobParameter>
    },

    #[serde(rename = "hudson.plugins.jira.JiraProjectProperty")]
    Project,

    #[serde(rename = "hudson.plugins.throttleconcurrents.ThrottleJobProperty")]
    Concurrency,

    #[serde(rename = "hudson.plugins.jobConfigHistory.JobLocalConfiguration")]
    Local,
}


#[derive(Deserialize, Debug)]
struct JenkinsJobInfoResponse{
    property: Vec<JenkinsJobInfoProperty>
}

#[derive(Debug)]
pub enum InfoRequestError{
    Request(reqwest::Error),
    NoParams
}

// Для прозрачной ковертации ошибки
impl From<reqwest::Error> for InfoRequestError {
    fn from(err: reqwest::Error) -> Self {
        InfoRequestError::Request(err)
    }
}

/// Запрашиваем список возможных
pub async fn request_jenkins_job_info(client: &reqwest::Client, auth: &JenkinsAuth, job_name: &str) -> Result<Vec<JenkinsJobParameter>, InfoRequestError> {
    // https://jenkins.17btest.com/job/bbd-gplay-prod/api/json?pretty=true

    // debug!("{} {}", jenkins_user, jenkins_token);

    let response = {
        let job_info_url = format!("https://jenkins.17btest.com/job/{}/api/json", job_name);
        client
            .get(job_info_url.as_str())
            .basic_auth(&auth.jenkins_user, Some(&auth.jenkins_api_token))
            .send()
            .await?
    };

    let result = response
        .json::<JenkinsJobInfoResponse>()
        .await?;

    let parameters: Option<Vec<JenkinsJobParameter>> = result
        .property
        .into_iter()
        .find_map(|property|{
            match property {
                JenkinsJobInfoProperty::Parameter{parameters} => {
                    Some(parameters)
                },
                _ => {
                    None
                }
            }
        });

    debug!("Job info result: {:?}", parameters);
    
    parameters.ok_or(InfoRequestError::NoParams)
}*/