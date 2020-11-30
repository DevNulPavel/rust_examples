use serde::{
    Deserialize
};
use serde_json::{
    Value
};
use reqwest::{
    Client
};
use log::{
    debug,
    info
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
    client: Client,
    jenkins_user: String,
    jenkins_api_token: String,
    info: JenkinsJobInfo
}

impl<'a> JenkinsJob {
    pub fn new(client: Client, jenkins_user: &str, jenkins_api_token: &str, info: JenkinsJobInfo) -> JenkinsJob {
        JenkinsJob{
            client,
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
            let response = {
                let job_info_url = format!("https://jenkins.17btest.com/job/{}/config.xml", self.info.name);
                self.client
                    .get(job_info_url.as_str())
                    .basic_auth(&self.jenkins_user, Some(&self.jenkins_api_token))
                    .send()
                    .await
                    .map_err(|err|{
                        JenkinsError::RequestError(err, format!("Target config request error: {}", job_info_url))
                    })?
            };

            let xml_data = response
                .text()
                .await
                .map_err(|err|{
                    JenkinsError::BodyParseError(err, format!("Target config parse error"))
                })?;
            
            debug!("Job parameters info: {}", xml_data);

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

    pub async fn start_job(&self, parameters: Value) -> Result<String, JenkinsError> {
        // https://jenkins.17btest.com/job/utils-check-free-space/api/
        // https://jenkins.17btest.com/job/utils-check-free-space/buildWithParameters
        // https://coderoad.ru/51508222/%D0%9A%D0%B0%D0%BA%D0%BE%D0%B2-%D1%84%D0%BE%D1%80%D0%BC%D0%B0%D1%82-JSON-%D0%B4%D0%BB%D1%8F-Jenkins-REST-buildWithParameters-%D1%87%D1%82%D0%BE%D0%B1%D1%8B-%D0%BF%D0%B5%D1%80%D0%B5%D0%BE%D0%BF%D1%80%D0%B5%D0%B4%D0%B5%D0%BB%D0%B8%D1%82%D1%8C

        let response = {
            let job_build_url = format!("https://jenkins.17btest.com/job/{}/buildWithParameters", self.info.name);
            self.client
                .post(job_build_url.as_str())
                .basic_auth(&self.jenkins_user, Some(&self.jenkins_api_token))
                .form(&parameters)
                .send()
                .await
                .map_err(|err|{
                    JenkinsError::RequestError(err, format!("Build job with params error: {}", job_build_url))
                })?
            };

        // reqwest::StatusCode::from_u16(201).unwrap()
        if response.status() != http::StatusCode::CREATED {
            return Err(JenkinsError::LogicalError(format!("Job {} start failed", self.info.name)));
        }

        // Получаем урл на итем сборки
        let build_info_url = response
            .headers()
            .get(http::header::LOCATION)
            .ok_or_else(||{
                JenkinsError::LogicalError(format!("Job {} start failed, there is no URL", self.info.name))
            })?
            .to_str()
            .map_err(|_|{
                JenkinsError::LogicalError(format!("Job {} start failed, URL parse failed", self.info.name))
            })?;

        // https://jenkins.17btest.com/queue/item/23115/api/json?pretty=true
        #[derive(Deserialize, Debug)]
        struct ItemInfoTask{
            url: String
        }
        #[derive(Deserialize, Debug)]
        struct ItemInfoResponse{
            task: ItemInfoTask,
            executable: Option<ItemInfoTask>,
        }

        // Запрос информации о сборке
        let item_info_response: ItemInfoResponse = {
            let build_info_url = format!("{}api/json", build_info_url);

            debug!("Jenkins build task info url: {}", build_info_url);

            self.client
                .get(build_info_url.as_str())
                .basic_auth(&self.jenkins_user, Some(&self.jenkins_api_token))
                .send()
                .await
                .map_err(|err|{
                    JenkinsError::RequestError(err, format!("Job info request error: {}", build_info_url))
                })?
                .json::<ItemInfoResponse>()
                .await
                .map_err(|err|{
                    JenkinsError::JsonParseError(err, format!("Job info parse error: {}", build_info_url))
                })?
        };

        let url = if let Some(info) = item_info_response.executable {
            info.url
        }else{
            item_info_response.task.url
        };

        // TODO: тут можем запустить мониторинг статуса нашей задачи по урлу

        info!("New job {} started: {}", self.info.name, url); // Check queue

        Ok(url)
    }
}