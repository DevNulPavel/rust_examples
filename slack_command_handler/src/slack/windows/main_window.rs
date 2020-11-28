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
        JenkinsClient,
        JenkinsJob
    },
    application_data::{
        ApplicationData
    },
    slack::{
        view::{
            View
        },
        // view_open_response::{
        //     ViewOpenResponse,
        //     ViewUpdateResponse,
        //     ViewInfo
        // }
    }
};
use super::{
    properties_window::{
        open_build_properties_window_by_reponse
    },
    parameters::{
        WindowParametersPayload,
        WindowParameters
    }
};


fn window_json_with_jobs(jobs: Option<Vec<JenkinsJob>>) -> Value {
    let options_json = match jobs {
        Some(array) => {
            // Конвертируем джобы в Json
            let jobs_json = array.into_iter()
                .map(|job|{
                    //debug!("Job info: {:?}", job);
                    serde_json::json!(
                        {
                            "text": {
                                "type": "plain_text",
                                "text": job.get_info().name,
                                "emoji": false
                            },
                            "value": job.get_info().name
                        }
                    )
                })
                .collect::<Vec<serde_json::Value>>();

            serde_json::json!([
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
                }
            ])
        },
        None => {
            serde_json::json!([
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": "Loading"
                    }
                }
            ])
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
        "blocks": options_json,
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
pub async fn open_main_build_window(app_data: Data<ApplicationData>, trigger_id: String) {
    // Выполняем наш запрос
    // TODO: Вернуть ошибку
    info!("Open main window with trigger_id: {}", trigger_id);

    let window_view = window_json_with_jobs(None);

    let window = serde_json::json!({
        "trigger_id": trigger_id,
        "view": window_view
    });

    let ApplicationData{slack_client, jenkins_client} = app_data.as_ref();
    let open_result = slack_client
        .open_view(window)
        .await;
    
    match open_result {
        Ok(view) => {
            // Запускаем асинхронный запрос, чтобы моментально ответить
            // Иначе долгий запрос отвалится по таймауту
            //update_main_window(view, jenkins_auth).await;
        },
        Err(err) => {
            error!("Main window open error: {:?}", err);
        }
    }
}

async fn update_main_window<'a>(view: View<'a>, jenkins: &JenkinsClient){
    info!("Main window view update");

    // Запрашиваем список джобов
    let jobs = match jenkins.request_jenkins_jobs_list().await {
        Ok(jobs) => {
            jobs
        },
        Err(err) => {
            error!("Jobs request failed: {:?}", err);
            return;
            // TODO: Написать ошибочное в ответ
        }
    };

    // Описываем обновление нашего окна
    // https://api.slack.com/surfaces/modals/using#interactions
    let window_view = window_json_with_jobs(Some(jobs));

    let update_result = view
        .update_view(window_view)
        .await;

    match update_result {
        Ok(view) => {
        },
        Err(err) => {
            error!("Main window update error: {:?}", err);
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
            info!("Parsed payload: {:?}", payload);

            // Обрабатываем команды окна, запуск происходит асинхронно, 
            // чтобы максимально быстро ответить серверу
            // https://api.slack.com/surfaces/modals/using#interactions
            spawn(async move {
                // TODO: Хрень полная, но больше никак не удостовериться, что сервер слака получил ответ обработчика
                actix_web::rt::time::delay_for(std::time::Duration::from_millis(200)).await;

                match payload {
                    // Вызывается на нажатие кнопки подтверждения
                    WindowParametersPayload::Submit{view, trigger_id} => {
                        debug!("Submit button processing with trigger_id: {}", trigger_id);
            
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
            });

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