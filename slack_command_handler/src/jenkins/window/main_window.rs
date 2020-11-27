use log::{
    info,
    debug,
    error
};
// use futures::{
//     FutureExt
// };
use actix_web::{ 
    rt::{
        spawn
    },
    web::{
        Form,
        Data
    },
    HttpResponse
};
use serde_json::{
    Value
};
use crate::{
    jenkins::{
        api::{
            request_jenkins_jobs_list,
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
        WindowParameters
    },
    view_open_response::{
        ViewOpenResponse,
        ViewUpdateResponse,
        ViewInfo
    }
};

fn window_json_with_jobs(jobs: Option<Vec<Value>>) -> Value {
    let options_json = match jobs {
        Some(array) => {
            serde_json::json!({
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
                    "options": array
                },
                "label": {
                    "type": "plain_text",
                    "text": "Target",
                    "emoji": false
                }
            })
        },
        None => {
            serde_json::json!({
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": "Loading"
                }
            })
        }
    };

    // Описываем наше окно
    serde_json::json!({
        "type": "modal",
        "callback_id": "build_jenkins_id",
        "title": {
            "type": "plain_text",
            "text": "Build jenkins target",
            "emoji": false
        },
        "blocks": [         
            options_json
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
    })
}

/// Обработчик открытия окна Jenkins
pub async fn open_main_build_window(app_data: Data<ApplicationData>, trigger_id: &str) {
    // Выполняем наш запрос
    // TODO: Вернуть ошибку
    info!("Open main window with trigger_id: {}", trigger_id);

    let window_view = window_json_with_jobs(None);

    let window = serde_json::json!({
        "trigger_id": trigger_id,
        "view": window_view
    });

    let response = app_data
        .http_client
        .post("https://slack.com/api/views.open")
        .bearer_auth(app_data.slack_api_token.as_str())
        .header("Content-type", "application/json")
        .body(serde_json::to_string(&window).unwrap())
        .send()
        .await;
    
    let response = match response {
        Ok(res) => res,
        Err(err) => {
            error!("Main window open response error: {:?}", err);
            return;
        }
    };

    let parsed = match response.json::<ViewOpenResponse>().await {
        Ok(parsed) => parsed,
        Err(err) => {
            error!("Response parse error: {}", err);
            return;
        }
    };

    match parsed {
        ViewOpenResponse::Ok{view} => {
            // Запускаем асинхронный запрос, чтобы моментально ответить
            // Иначе долгий запрос отвалится по таймауту
            spawn(update_main_window(view, app_data));
        },
        ViewOpenResponse::Error{error, ..}=>{
            error!("Response error: {}", error);
            return;
        }
    }
}

async fn update_main_window(view_info: ViewInfo, app_data: Data<ApplicationData>){
    info!("Main window view update");

    // Запрашиваем список джобов
    let jobs = match request_jenkins_jobs_list(&app_data.http_client, &app_data.jenkins_auth).await {
        Ok(jobs) => {
            jobs
        },
        Err(err) => {
            error!("Jobs request failed: {:?}", err);
            return;
            // TODO: Написать ошибочное в ответ
        }
    };

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

    // Описываем обновление нашего окна
    // https://api.slack.com/surfaces/modals/using#interactions
    let window_view = window_json_with_jobs(Some(jobs_json));

    let window = serde_json::json!({
        "view_id": view_info.id,
        "hash": view_info.hash,
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
        ViewUpdateResponse::Error{error, ..}=>{
            error!("Response error: {}", error);
        }
    }
}

/// Обработчик открытия окна Jenkins
async fn process_main_window_payload(payload: WindowParametersPayload, app_data: Data<ApplicationData>) {
    match payload {
        // Вызывается на нажатие кнопки подтверждения
        WindowParametersPayload::Submit{view, trigger_id} => {
            debug!("Submit button processing");

            // process_submit_button()

            // Открываем окно с параметрами сборки
            open_build_properties_window_by_reponse(trigger_id, view, app_data).await;
        },

        // Вызывается на нажатие разных кнопок в самом меню
        // TODO: Можно делать валидацию ветки здесь
        WindowParametersPayload::Action{..} => {
            debug!("Action processing");

            //update_main_window(view, app_data).await;
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

            // Обрабатываем команды окна, запуск происходит асинхронно, 
            // чтобы максимально быстро ответить серверу
            // https://api.slack.com/surfaces/modals/using#interactions
            spawn(process_main_window_payload(payload, app_data));

            HttpResponse::Ok()
                .finish()
        },
        Err(err) => {
            error!("Payload parse error: {:?}", err);
            
            // TODO: Error
            HttpResponse::Ok()
                .body(format!("Payload parse error: {}", err))
        }
    }
}