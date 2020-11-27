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
    // HttpResponse
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
    slack::{
        view::{
            View
        }
    },
    ApplicationData
};
use super::{
    parameters::{
        WindowParametersViewInfo
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

    let view_open_res = app_data.slack_client
        .open_view(new_window)
        .await;

    match view_open_res {
        Ok(view) => {
            // Запускаем асинхронный запрос, чтобы моментально ответить
            // Иначе долгий запрос отвалится по таймауту
            //update_properties_window(selected_target, view, app_data).await
        },
        Err(err) => {
            error!("Properties window open response error: {:?}", err);
        }
    }
}

async fn update_properties_window(selected_target: String, view: View, app_data: Data<ApplicationData>) {
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

    let update_result = view
        .update_view(window_view)
        .await;

    match update_result {
        Ok(view) => {
        },
        Err(err) => {
            error!("Properties window update error: {:?}", err);
        }
    }
}