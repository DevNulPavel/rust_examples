use std::{
    fmt,
    collections::{
        HashMap
    }
};
use log::{
    debug,
    // info,
    error
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
use serde_json::{
    Value
};
use crate::{
    ApplicationData
};
use super::{
    api::{
        request_jenkins_job_info,
        Parameter
    }
};

// https://api.slack.com/reference/interaction-payloads/block-actions

#[derive(Deserialize, Serialize, Debug)]
pub struct SlackWindowState{
    values: HashMap<String, Value>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SlackWindowParametersViewInfo{
    id: String,
    hash: String,
    state: SlackWindowState

    // Прочие поля
    // #[serde(flatten)]
    // other: HashMap<String, Value>
}

// https://serde.rs/enum-representations.html
// https://api.slack.com/reference/interaction-payloads
#[derive(Deserialize, Serialize)]
#[serde(tag = "type")] // Вариант enum будет выбираться по полю type, значения переименовываются
pub enum SlackWindowParametersPayload{
    #[serde(rename = "view_submission")]
    Submit{
        view: SlackWindowParametersViewInfo,
    },
    #[serde(rename = "block_actions")]
    Action{
        trigger_id: String,
        response_url: Option<String>,
        view: SlackWindowParametersViewInfo,
        actions: Vec<Value>,
    }

    // pub user: HashMap<String, serde_json::Value>,
    // pub view: HashMap<String, Value>,

    // Прочие поля
    // #[serde(flatten)]
    // other: HashMap<String, Value>
}

impl fmt::Debug for SlackWindowParametersPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(serde_json::to_string_pretty(self)
            .unwrap()
            .as_str())
    }
}

#[derive(Deserialize, Debug)]
pub struct SlackWindowParameters{
    pub payload: String
}

fn process_submit_button() -> web::HttpResponse{
    // TODO: Не конвертировать туда-сюда json
    // let j = r#"
    //     {
    //     "id": "demo-deserialize-max",
    //     "values": [
    //         256,
    //         100,
    //         384,
    //         314,
    //         271
    //     ]
    //     }
    // "#;
    let window_update = serde_json::json!(
        {
            "response_action": "update",
            "view": {
                "type": "modal",
                "title": {
                    "type": "plain_text",
                    "text": "Updated view"
                },
                "blocks": [
                    {
                        "type": "section",
                        "text": {
                            "type": "plain_text",
                            "text": "I've changed and I'll never be the same. You must believe me."
                        }
                    }
                ]
            }
        }                        
    );

    HttpResponse::Ok()
        .header("Content-type", "application/json")
        .body(serde_json::to_string(&window_update).unwrap())
}

// https://api.slack.com/surfaces/modals/using
async fn open_build_properties_window_by_reponse(view: SlackWindowParametersViewInfo, app_data: web::Data<ApplicationData>) -> web::HttpResponse{
    
    // Получаем из недр Json нужный нам элемент
    let selected_target = {
        let selected_target = view
            .state
            .values
            .get("build_target_block_id")
            .and_then(|val|{
                val.get("build_target_action_id")
            })
            .and_then(|val|{
                val.get("selected_option")
            })
            .and_then(|val|{
                val.get("value")
            })
            .and_then(|val|{
                val.as_str()
            });

        match selected_target {
            Some(target) => {
                target
            },
            None =>{
                // TODO: Error
                error!("Select target error");
                return HttpResponse::Ok()
                    .body(format!("Select target error"))
            }
        }
    };

    // Запрашиваем список параметров
    let parameter_blocks: Vec<Value> = match request_jenkins_job_info(&app_data.http_client, 
                                   &app_data.jenkins_auth,
                                   selected_target).await{
        Ok(parameters) => {
            debug!("Parameters list: {:?}", parameters);

            parameters
                .into_iter()
                .map(|param|{
                    //debug!("Job info: {:?}", job);

                    // Примеры компонентов
                    // https://app.slack.com/block-kit-builder/
                    match param {
                        Parameter::Boolean{name, description, default_value} => {
                            /*serde_json::json!({
                                "type": "input",
                                "element": {
                                    "type": "checkboxes",
                                    "options": [
                                        {
                                            "text": {
                                                "type": "plain_text",
                                                "text": name,
                                                "emoji": true
                                            },
                                            "value": "value-0"
                                        }
                                    ]
                                }
                            })*/
                            serde_json::json!({
                                "type": "section",
                                "text": {
                                    "type": "plain_text",
                                    "text": name,
                                    "emoji": true
                                }
                            })
                        },
                        Parameter::Choice{name, ..} => {
                            serde_json::json!({
                                "type": "section",
                                "text": {
                                    "type": "plain_text",
                                    "text": name,
                                    "emoji": true
                                }
                            })
                        },
                        Parameter::Git{name, ..} => {
                            serde_json::json!({
                                "type": "section",
                                "text": {
                                    "type": "plain_text",
                                    "text": name,
                                    "emoji": true
                                }
                            })
                        },
                        Parameter::String{name, description, default_value} => {
                            /*serde_json::json!({
                                "type": "input",
                                "element": {
                                    "type": "plain_text_input"
                                },
                                "label": {
                                    "type": "plain_text",
                                    "text": name,
                                    "emoji": true
                                }
                            })*/
                            serde_json::json!({
                                "type": "section",
                                "text": {
                                    "type": "plain_text",
                                    "text": name,
                                    "emoji": true
                                }
                            })
                        }
                    }
                })
                .collect::<Vec<serde_json::Value>>()
        },
        Err(err) => {
           error!("Job info request error: {:?}", err);
            return HttpResponse::Ok()
                .body(format!("Select target error: {:?}", err));
        }
    };

    // TODO: Не конвертировать туда-сюда json
    // let j = r#"
    //     {
    //     "id": "demo-deserialize-max",
    //     "values": [
    //     ]
    //     }
    // "#;
    let new_window = serde_json::json!(
        {
            "response_action": "push",
            "view": {
                "type": "modal",
                "title": {
                    "type": "plain_text",
                    "text": "Updated view"
                },
                "blocks": parameter_blocks
            }
        }                    
    );

    HttpResponse::Ok()
        .header("Content-type", "application/json")
        .body(serde_json::to_string(&new_window).unwrap())
}

async fn update_window(view: SlackWindowParametersViewInfo, app_data: web::Data<ApplicationData>) -> web::HttpResponse{
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

async fn push_new_window(trigger_id: &str, app_data: web::Data<ApplicationData>) -> web::HttpResponse{
    let new_window = serde_json::json!(
        {
            "trigger_id": trigger_id,
            "view": {
                "type": "modal",
                "callback_id": "edit-task",
                "title": {
                "type": "plain_text",
                "text": "Edit task details"
                },
                "submit": {
                    "type": "plain_text",
                    "text": "Create"
                },
                "blocks": [
                {
                    "type": "input",
                    "block_id": "edit-task-title",
                    "label": {
                    "type": "plain_text",
                    "text": "Task title"
                    },
                    "element": {
                    "type": "plain_text_input",
                    "action_id": "task-title-value",
                    "initial_value": "Block Kit documentation"
                    },
                },
                {
                    "type": "input",
                    "block_id": "edit-ticket-desc",
                    "label": {
                    "type": "plain_text",
                    "text": "Ticket description"
                    },
                    "element": {
                    "type": "plain_text_input",
                    "multiline": true,
                    "action_id": "ticket-desc-value",
                    "initial_value": "Update Block Kit documentation to include Block Kit in new surface areas (like modals)."
                    }
                }
                ]
            }
            }
    );

    // Выполняем запрос обновления вьюшки
    let response = app_data
        .http_client
        .post("https://slack.com/api/views.push")
        .bearer_auth(app_data.slack_api_token.as_str())
        .header("Content-type", "application/json")
        .body(serde_json::to_string(&new_window).unwrap())
        .send()
        .await;
    
    match response {
        Ok(res) => {
            debug!("Window create response: {:?}", res);
            HttpResponse::Ok()
                .finish()    
        },
        Err(err) => {
            error!("Window create error: {:?}", err);
            // TODO: Error
            HttpResponse::Ok()
                .body(format!("Window create error: {}", err))
        }
    }
}

async fn process_payload(payload: SlackWindowParametersPayload, app_data: web::Data<ApplicationData>) -> web::HttpResponse{
    match payload {
        // Вызывается на нажатие кнопки подтверждения
        SlackWindowParametersPayload::Submit{view} => {
            // process_submit_button()
            open_build_properties_window_by_reponse(view, app_data).await
        },

        // Вызывается на нажатие разных кнопок в самом меню
        // TODO: Можно делать валидацию ветки здесь
        SlackWindowParametersPayload::Action{view, ..} => {
            update_window(view, app_data).await
            // push_new_window
        }
    }  
}

// std::collections::HashMap<String, serde_json::Value>
pub async fn jenkins_window_handler(parameters: web::Form<SlackWindowParameters>, app_data: web::Data<ApplicationData>) -> web::HttpResponse {
    //debug!("Jenkins window parameters: {:?}", parameters);

    match serde_json::from_str::<SlackWindowParametersPayload>(parameters.payload.as_str()){
        Ok(payload) => {
            debug!("Parsed payload: {:?}", payload);

            // Обрабатываем команды окна
            // https://api.slack.com/surfaces/modals/using#interactions
            process_payload(payload, app_data).await
        },
        Err(err) => {
            error!("Payload parse error: {:?}", err);
            // TODO: Error
            HttpResponse::Ok()
                .body(format!("Payload parse error: {}", err))
        }
    }
}