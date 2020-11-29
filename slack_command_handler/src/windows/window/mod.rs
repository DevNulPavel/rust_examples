/*mod view_open_response;

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
use crate::{
    ApplicationData
};*/


// https://api.slack.com/reference/interaction-payloads/block-actions

/*fn process_submit_button() -> web::HttpResponse{
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
}*/

