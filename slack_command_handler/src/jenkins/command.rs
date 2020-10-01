use std::{
    fmt
};
use actix_web::{ 
    web,
    // Responder,
    HttpResponse
};
use serde::{
    Serialize,
    Deserialize
};
use log::{
    debug,
    // info,
    error
};
use super::{
    api::{
        request_jenkins_jobs_list,
        request_jenkins_job_info
    }
};
use crate::{
    ApplicationData
};


#[derive(Deserialize, Serialize)]
pub struct SlackCommandParameters{
    user_id: String,
    user_name: String,
    trigger_id: String,
    command: String,

    // pub token: String,
    // pub text: String,
    // channel_id: String,
    // team_id: String,
    // team_domain: String,
    // enterprise_id: String,
    // enterprise_name: String,
    // channel_name: String,
    // response_url: String,
    // api_app_id: String

    // Так можно получить прочие необязательные параметры
    // https://serde.rs/attr-flatten.html
    // #[serde(flatten)]
    // extra: HashMap<String, Value>
}

impl fmt::Debug for SlackCommandParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = serde_json::to_string_pretty(self).unwrap();
        f.write_str(text.as_str())
    }
}

pub async fn jenkins_command_handler(parameters: web::Form<SlackCommandParameters>, app_data: web::Data<ApplicationData>) -> web::HttpResponse {
    debug!("Index parameters: {:?}", parameters);

    match request_jenkins_job_info(&app_data.http_client, 
                                   &app_data.jenkins_auth,
                                   "bbd-gplay-prod").await{
        Ok(_res) => {

        },
        Err(err) => {
            error!("Job info request error: {:?}", err);
        }
    }

    // Получаем список возможных таргетов сборки
    // TODO: может можно избавиться от collect?
    let jobs = match request_jenkins_jobs_list(&app_data.http_client, 
                                               &app_data.jenkins_auth).await {
        Ok(jobs) => {
            jobs
                .into_iter()
                .map(|job|{
                    //debug!("Job info: {:?}", job);
                    serde_json::json!(
                        {
                            "text": {
                                "type": "plain_text",
                                "text": job.name,
                                "emoji": false
                            },
                            "value": job.name
                        }
                    )
                })
                .collect::<Vec<serde_json::Value>>()
        },
        Err(err) => {
            error!("Jobs request failed: {:?}", err);
            return HttpResponse::Ok()
                        .body(format!("{:?}", err));
        }
    };
    
    // Описываем наше окно
    let window = serde_json::json!(
        {
            "trigger_id": parameters.trigger_id,
            "view": {
                "type": "modal",
                "callback_id": "build_jenkins_id",
                "title": {
                    "type": "plain_text",
                    "text": "Build jenkins target",
                    "emoji": false
                },
                "blocks": [
                    {
                        "type": "input",
                        "element": {
                            "type": "static_select",
                            "placeholder": {
                                "type": "plain_text",
                                "text": "Select or type build target",
                                "emoji": false
                            },
                            "options": jobs
                        },
                        "label": {
                            "type": "plain_text",
                            "text": "Target",
                            "emoji": false
                        }
                    },
                    {
                        "type": "input",
                        "element": {
                            "type": "plain_text_input"
                        },
                        "label": {
                            "type": "plain_text",
                            "text": "Git branch",
                            "emoji": false
                        }
                    },             
                    {
                        "type": "section",
                        "block_id": "section-identifier",
                        "text": {
                            "type": "mrkdwn",
                            "text": "Test markdown text"
                        },
                        "accessory": {
                            "type": "button",
                            "action_id": "test_button_id",
                            "text": {
                                "type": "plain_text",
                                "text": "Test button"
                            }
                        }
                    }
                ],
                "submit": {
                    "type": "plain_text",
                    "text": "Submit",
                    "emoji": false
                },
                "close": {
                    "type": "plain_text",
                    "text": "Cancel",
                    "emoji": false
                }
            }
        }
    );

    // Выполняем наш запрос
    let response = app_data
        .http_client
        .post("https://slack.com/api/views.open")
        .bearer_auth(app_data.slack_api_token.as_str())
        .header("Content-type", "application/json")
        .body(serde_json::to_string(&window).unwrap())
        .send()
        .await;
        
    match response {
        Ok(res) => {
            debug!("Window open result: {:?}", res);
            HttpResponse::Ok()
                .finish()
            // let response = SlackCommandResponse{
            //     response_type: "ephemeral",
            //     text: String::from("test")
            // };
        
            // HttpResponse::Ok()
            //     .json(response)            
        },
        Err(err) =>{
            error!("Window open error: {:?}", err);
            HttpResponse::Ok()
                .body(format!("{:?}", err))
        }
    }
}
