use log::{
    debug
};
use serde::{
    Deserialize
};
use super::{
    error::{
        JenkinsError
    },
    request_builder::{
        JenkinsRequestBuilder
    }
};

pub type JobUrl = String;

pub struct JenkinsJob{
    client: JenkinsRequestBuilder,
    info_api_url: String,
    job_url: Option<String>,
}

impl JenkinsJob {
    pub fn new(client: JenkinsRequestBuilder, url: &str) -> JenkinsJob {
        JenkinsJob{
            client,
            info_api_url: format!("{}api/json", url),
            job_url: None,
        }
    }

    pub fn get_info_api_url(&self) -> &String{
        &self.info_api_url
    }

    pub async fn try_to_get_real_job_url(&mut self) -> Result<Option<JobUrl>, JenkinsError>{
        if self.job_url.is_some(){
            return Ok(self.job_url.clone());
        }

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

        debug!("Request real job url: {}", self.info_api_url);

        // Запрос информации о сборке
        let item_info_response: ItemInfoResponse = self.client
            .build_get_request(self.info_api_url.as_str())
            .send()
            .await
            .map_err(|err|{
                JenkinsError::RequestError(err, format!("Job info request error: {}", self.info_api_url))
            })?
            .json::<ItemInfoResponse>()
            .await
            .map_err(|err|{
                JenkinsError::JsonParseError(err, format!("Job info parse error: {}", self.info_api_url))
            })?;

        if let Some(info) = item_info_response.executable {
            self.job_url = Some(info.url);
        }

        Ok(self.job_url.clone())
    }
}