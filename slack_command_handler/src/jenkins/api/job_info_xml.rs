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

#[derive(Deserialize, Debug)]
pub struct ChoiseList{
    #[serde(rename = "string")]
    values: Vec<String>
}

#[derive(Deserialize, Debug)]
pub struct Choise{
    #[serde(rename = "defaultChoice")]
    default_value: String,

    #[serde(rename = "choiceList")]
    choice_list: ChoiseList
}

#[derive(Deserialize, Debug)]
pub enum Parameter{
    #[serde(rename = "hudson.model.BooleanParameterDefinition")]
    Boolean{
        name: String,
        description: String,
    
        #[serde(rename = "defaultValue")]
        default_value: bool
    },
    #[serde(rename = "hudson.model.StringParameterDefinition")]
    String{
        name: String,
        description: String,

        #[serde(rename = "defaultValue")]
        default_value: String
    },
    #[serde(rename = "jp.ikedam.jenkins.plugins.extensible__choice__parameter.ExtensibleChoiceParameterDefinition")]
    Choice{
        name: String,
        description: String,
        
        #[serde(rename = "choiceListProvider")]
        choise: Choise,
    }
}

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

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum InfoRequestError{
    RequestErr(reqwest::Error),
    ParseErr(serde_xml_rs::Error),
    NoParams
}

// Для прозрачной ковертации ошибки
impl From<reqwest::Error> for InfoRequestError {
    fn from(err: reqwest::Error) -> Self {
        InfoRequestError::RequestErr(err)
    }
}

// Для прозрачной ковертации ошибки
impl From<serde_xml_rs::Error> for InfoRequestError {
    fn from(err: serde_xml_rs::Error) -> Self {
        InfoRequestError::ParseErr(err)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Запрашиваем список возможных
pub async fn request_jenkins_job_info(client: &reqwest::Client, auth: &JenkinsAuth, job_name: &str) -> Result<Vec<Parameter>, InfoRequestError> {
    // https://jenkins.17btest.com/job/bbd-gplay-prod/config.xml

    // debug!("{} {}", jenkins_user, jenkins_token);

    let result: InfoResponse = {
        let response = {
            let job_info_url = format!("https://jenkins.17btest.com/job/{}/config.xml", job_name);
            client
                .get(job_info_url.as_str())
                .basic_auth(&auth.jenkins_user, Some(&auth.jenkins_api_token))
                .send()
                .await?
        };

        let xml_text = response
            .text()
            .await?;

        serde_xml_rs::from_str(xml_text.as_str())?
    };

    let parameters: Vec<Parameter> = result
        .properties
        .parameters
        .parameters
        .values;

    debug!("Job info result: {:?}", parameters);
    
    Ok(parameters)

    // debug!("Job info result: {:?}", result);

    // Err(InfoRequestError::NoParams)
}