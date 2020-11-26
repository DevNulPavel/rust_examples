use log::{
    debug,
    error
};
use actix_web::{ 
    web::{
        Form,
        Data
    },
    HttpResponse
};
use crate::{
    jenkins::{
        api::{
            JenkinsJob
        }
    },
    application_data::{
        ApplicationData
    }
};
use super::{
    properties_window::{
        open_build_properties_window_by_reponse
    },
    parameters::{
        WindowParametersPayload,
        WindowParametersViewInfo,
        WindowState,
        WindowParameters
    }
};


/// Обработчик открытия окна Jenkins
pub async fn open_main_build_window(app_data: &ApplicationData, trigger_id: &str, jobs: Vec<JenkinsJob>) -> HttpResponse {
    // Конвертируем джобы в Json
    let jobs_json = jobs.into_iter()
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
        .collect::<Vec<serde_json::Value>>();

    // Описываем наше окно
    let window = serde_json::json!(
        {
            "trigger_id": trigger_id,
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
                        "block_id": "build_target_block_id",
                        "type": "input",
                        "element": {
                            "action_id": "build_target_action_id",
                            "type": "static_select",
                            "placeholder": {
                                "type": "plain_text",
                                "text": "Select or type build target",
                                "emoji": false
                            },
                            "options": jobs_json
                        },
                        "label": {
                            "type": "plain_text",
                            "text": "Target",
                            "emoji": false
                        }
                    },         
                    /*{
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
                    }*/
                ],
                "submit": {
                    "type": "plain_text",
                    "text": "To build properties",
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

async fn update_main_window(view: WindowParametersViewInfo, app_data: Data<ApplicationData>) -> HttpResponse{
    // Описываем обновление нашего окна
    // https://api.slack.com/surfaces/modals/using#interactions
    let window_update = serde_json::json!(
        {
            "view_id": view.id,
            "hash": view.hash,
            "view": {
                "type": "modal",
                "callback_id": "view-helpdesk",
                "title": {
                    "type": "plain_text",
                    "text": "Submit an issue"
                },
                "submit": {
                    "type": "plain_text",
                    "text": "Submit"
                },
                "blocks": [
                    {
                        "type": "input",
                        "block_id": "ticket-title",
                        "label": {
                            "type": "plain_text",
                            "text": "Ticket title"
                        },
                        "element": {
                            "type": "plain_text_input",
                            "action_id": "ticket-title-value"
                        }
                    },
                    {
                        "type": "input",
                        "block_id": "ticket-desc",
                        "label": {
                            "type": "plain_text",
                            "text": "Ticket description"
                        },
                        "element": {
                            "type": "plain_text_input",
                            "multiline": true,
                            "action_id": "ticket-desc-value"
                        }
                    }
                ]
            }
        }
    );

    // Выполняем запрос обновления вьюшки
    let response = app_data
        .http_client
        .post("https://slack.com/api/views.update")
        .bearer_auth(app_data.slack_api_token.as_str())
        .header("Content-type", "application/json")
        .body(serde_json::to_string(&window_update).unwrap())
        .send()
        .await;
    
    match response {
        Ok(res) => {
            debug!("Window modify response: {:?}", res);
            HttpResponse::Ok()
                .finish()    
        },
        Err(err) => {
            error!("Window modify error: {:?}", err);
            // TODO: Error
            HttpResponse::Ok()
                .body(format!("Window modify error: {}", err))
        }
    }    
}

/// Обработчик открытия окна Jenkins
async fn process_main_window_payload(payload: WindowParametersPayload, app_data: Data<ApplicationData>) -> HttpResponse {
    match payload {
        // Вызывается на нажатие кнопки подтверждения
        WindowParametersPayload::Submit{view} => {
            debug!("Submit button processing");

            // process_submit_button()

            // Открываем окно с параметрами сборки
            open_build_properties_window_by_reponse(view, app_data).await
        },

        // Вызывается на нажатие разных кнопок в самом меню
        // TODO: Можно делать валидацию ветки здесь
        WindowParametersPayload::Action{view, ..} => {
            debug!("Action processing");

            update_main_window(view, app_data).await
            // push_new_window
        }
    }  
}

/// Обработчик открытия окна Jenkins
pub async fn main_build_window_handler(parameters: Form<WindowParameters>, app_data: Data<ApplicationData>) -> HttpResponse {
    debug!("Jenkins window parameters: {:?}", parameters);

    // Парсим переданные данные
    match serde_json::from_str::<WindowParametersPayload>(parameters.payload.as_str()){
        // Распарсили без проблем
        Ok(payload) => {
            debug!("Parsed payload: {:?}", payload);

            // Обрабатываем команды окна
            // https://api.slack.com/surfaces/modals/using#interactions
            process_main_window_payload(payload, app_data).await
        },
        Err(err) => {
            error!("Payload parse error: {:?}", err);
            
            // TODO: Error
            HttpResponse::Ok()
                .body(format!("Payload parse error: {}", err))
        }
    }
}