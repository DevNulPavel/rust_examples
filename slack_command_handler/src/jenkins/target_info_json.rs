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
pub struct JenkinsTargetParameterDefaultBoolValue{
    pub name: String,
    pub value: bool
}

#[derive(Deserialize, Debug)]
pub struct JenkinsTargetParameterDefaultStringValue{
    pub name: String,
    pub value: String
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum JenkinsTargetParameter{
    #[serde(rename = "BooleanParameterDefinition")]
    Boolean{
        name: String,
        description: String,
    
        #[serde(rename = "defaultParameterValue")]
        default_value: JenkinsTargetParameterDefaultBoolValue
    },
    #[serde(rename = "StringParameterDefinition")]
    String{
        name: String,
        description: String,
    
        #[serde(rename = "defaultParameterValue")]
        default_value: JenkinsTargetParameterDefaultStringValue
    },
    #[serde(rename = "ExtensibleChoiceParameterDefinition")]
    Choice{
        name: String,
        description: String,
    
        #[serde(rename = "defaultParameterValue")]
        default_value: JenkinsTargetParameterDefaultStringValue
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "_class")]
enum JenkinsTargetInfoProperty{
    #[serde(rename = "hudson.model.ParametersDefinitionProperty")]
    Parameter{
        #[serde(rename = "parameterDefinitions")]
        parameters: Vec<JenkinsTargetParameter>
    },

    #[serde(rename = "hudson.plugins.jira.JiraProjectProperty")]
    Project,

    #[serde(rename = "hudson.plugins.throttleconcurrents.ThrottleJobProperty")]
    Concurrency,

    #[serde(rename = "hudson.plugins.jobConfigHistory.JobLocalConfiguration")]
    Local,
}


#[derive(Deserialize, Debug)]
struct JenkinsTargetInfoResponse{
    property: Vec<JenkinsTargetInfoProperty>
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
pub async fn request_jenkins_job_info(client: &reqwest::Client, auth: &JenkinsAuth, job_name: &str) -> Result<Vec<JenkinsTargetParameter>, InfoRequestError> {
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
        .json::<JenkinsTargetInfoResponse>()
        .await?;

    let parameters: Option<Vec<JenkinsTargetParameter>> = result
        .property
        .into_iter()
        .find_map(|property|{
            match property {
                JenkinsTargetInfoProperty::Parameter{parameters} => {
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