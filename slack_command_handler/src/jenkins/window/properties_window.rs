use log::{
    debug,
    info,
    error
};
use actix_web::{ 
    web::{
        Data
    },
    rt::{
        spawn
    },
    // Responder,
    HttpResponse
};
use serde_json::{
    Value
};
use crate::{
    jenkins::{
        api::{
            request_jenkins_job_info,
            Parameter,
            ChoiseList,
            ChoiseInfo
        }
    },
    ApplicationData
};
use super::{
    parameters::{
        WindowParametersViewInfo
    },
    view_open_response::{
        ViewInfo,
        ViewOpenResponse,
        ViewUpdateResponse
    }
};

fn param_to_json_field(param: Parameter) -> Value {
    // Примеры компонентов
    // https://api.slack.com/reference/block-kit/block-elements
    // https://app.slack.com/block-kit-builder/
    match param {
        Parameter::Boolean{name, default_value, ..} => {
            let initial_selected_value = if default_value {
                serde_json::json!({
                    "value": "on",
                    "text": {
                        "type": "plain_text",
                        "text": "on"
                    }
                })
            }else{
                serde_json::json!({
                    "value": "off",
                    "text": {
                        "type": "plain_text",
                        "text": "off"
                    }
                })
            };

            serde_json::json!({
                "type": "section",
                "text": {
                    "type": "plain_text",
                    "text": name,
                    "emoji": true
                },
                "accessory": {
                    "type": "radio_buttons",
                    "action_id": "this_is_an_action_id",
                    "initial_option": initial_selected_value,
                    "options": [
                        {
                            "value": "on",
                            "text": {
                                "type": "plain_text",
                                "text": "on"
                            }
                        },
                        {
                            "value": "off",
                            "text": {
                                "type": "plain_text",
                                "text": "off"
                            }
                        }
                    ]
                }
            })
            /*serde_json::json!({
                "type": "section",
                "text": {
                    "type": "plain_text",
                    "text": name,
                    "emoji": true
                }
            })*/
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
            /*serde_json::json!({
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": name
                },
                "accessory": {
                    "type": "radio_buttons",
                    "options": [
                        {
                            "text": {
                                "type": "plain_text",
                                "text": "*this is plain_text text*",
                                "emoji": true
                            },
                            "value": "value-0"
                        }
                    ],
                    "action_id": "radio_buttons-action"
                }
            })*/
        },
        Parameter::ChoiceSimple{name, ..} => {
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
        Parameter::String{name, ..} => {
            serde_json::json!({
                "type": "input",
                //"block_id": name,
                "label": {
                    "type": "plain_text",
                    "text": name
                },
                "element": {
                    "type": "plain_text_input",
                    "action_id": name,
                    "placeholder": {
                        "type": "plain_text",
                        "text": "Enter some plain text"
                    }
                }
            })
        }
    }
}

fn create_window_view(params: Option<Vec<Parameter>>) -> Value {
    let blocks = match params {
        Some(params) => {
            // Параметры конвертируем в поля на окне
            params
                .into_iter()
                .map(|param|{
                    param_to_json_field(param)
                })
                .collect::<Vec<serde_json::Value>>()
        },
        None => {
            vec!(serde_json::json!({
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": "Loading"
                }
            }))
        }
    };

    serde_json::json!({
        "type": "modal",
        "callback_id": "modal-identifier_2",
        "title": {
            "type": "plain_text",
            "text": "Properties window"
        },
        "submit": {
            "type": "plain_text",
            "text": "Submit",
            "emoji": true
        },
        "close": {
            "type": "plain_text",
            "text": "Cancel",
            "emoji": true
        },
        "blocks": blocks
    })
}

/// Получаем из вьюшки имя нашего таргета
fn get_selected_target(view: &WindowParametersViewInfo) -> Option<&str>{
    view.state.values
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
        })
}

// https://api.slack.com/surfaces/modals/using
pub async fn open_build_properties_window_by_reponse(trigger_id: String, view: WindowParametersViewInfo, app_data: Data<ApplicationData>) {
    // https://api.slack.com/surfaces/modals/using#preparing_for_modals
    // Получаем из недр Json имя нужного нам таргета сборки
    let selected_target = {
        match get_selected_target(&view) {
            Some(target) => target.to_owned(),
            None =>{
                // TODO: Error
                error!("Select target error");
                return;
            }
        }        
    };

    // TODO: Не конвертировать туда-сюда json
    // let j = r#""#;
    let new_window = serde_json::json!({
        "trigger_id": trigger_id,
        "view": create_window_view(None)
    });

    // Выполняем наш запрос
    let response = app_data
        .http_client
        .post("https://slack.com/api/views.open")
        .bearer_auth(app_data.slack_api_token.as_str())
        .header("Content-type", "application/json")
        .body(serde_json::to_string(&new_window).unwrap())
        .send()
        .await;
    
    // Валидный ли ответ?
    let response = match response {
        Ok(res) => res,
        Err(err) => {
            error!("Properties window open response error: {:?}", err);
            return;
        }
    };

    // Парсим
    let parsed = match response.json::<ViewOpenResponse>().await {
        Ok(parsed) => parsed,
        Err(err) => {
            error!("Properties window open response parse error: {}", err);
            return;
        }
    };

    info!("Properties window open response: {:?}", parsed);

    // Обработка результата вьюшки
    match parsed {
        ViewOpenResponse::Ok{view} => {
            // Запускаем асинхронный запрос, чтобы моментально ответить
            // Иначе долгий запрос отвалится по таймауту
            update_properties_window(selected_target, view, app_data).await
        },
        err @ ViewOpenResponse::Error{..}=>{
            error!("Properties window open response error: {:?}", err);
            return;
        }
    }
}

async fn update_properties_window(selected_target: String, view: ViewInfo, app_data: Data<ApplicationData>) {
    // Запрашиваем список параметров данного таргета
    let parameters = match request_jenkins_job_info(&app_data.http_client, 
                                                    &app_data.jenkins_auth,
                                                    &selected_target).await{
        Ok(parameters) => {
            parameters
        },
        Err(err) => {
            // TODO: Error
            error!("Job info request error: {:?}", err);
            return;
        }
    };

    let window_view = create_window_view(Some(parameters));

    let window = serde_json::json!({
        "view_id": view.id,
        "hash": view.hash,
        "view": window_view
    });

    // Выполняем запрос обновления вьюшки
    let response = app_data
        .http_client
        .post("https://slack.com/api/views.update")
        .bearer_auth(app_data.slack_api_token.as_str())
        .header("Content-type", "application/json")
        .body(serde_json::to_string(&window).unwrap())
        .send()
        .await;  

    let response = match response{
        Ok(res) => res,
        Err(err) => {
            error!("Main window open response: {:?}", err);
            return;
        }
    }; 

    //debug!("Main window open response: {}", res.text().await.unwrap());  

    let parsed = match response.json::<ViewUpdateResponse>().await {
        Ok(parsed) => parsed,
        Err(err) => {
            error!("Response parse error: {}", err);
            return;
        }
    };

    debug!("Parsed response: {:?}", parsed);

    match parsed {
        ViewUpdateResponse::Ok{view} => {
            info!("Update success for view_id: {}", view.id);
        },
        error @ ViewUpdateResponse::Error{..}=>{
            error!("Response error: {:?}", error);
        }
    }
    //debug!("Parameters list: {:?}", parameters);
}