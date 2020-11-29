use log::{
    info,
    // debug,
    error
};
use async_trait::{
    async_trait
};
use actix_web::{ 
    // rt::{
        // spawn
    // },
    web::{
        // Form,
        Data
    },
    // HttpResponse
};
use serde_json::{
    Value
};
use crate::{
    jenkins::{
        // JenkinsClient,
        JenkinsJob
    },
    application_data::{
        ApplicationData
    },
    slack::{
        View,
        ViewInfo,
        ViewActionHandler,
        // SlackViewError
    }
};
use super::{
    properties_window::{
        open_build_properties_window_by_reponse
    }
};


const MAIN_WINDOW_ID: &str = "MAIN_WINDOW_ID";

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
        "callback_id": MAIN_WINDOW_ID,
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

    let open_result = app_data
        .slack_client
        .open_view(window)
        .await;
    
    match open_result {
        Ok(view) => {
            // Запускаем асинхронный запрос, чтобы моментально ответить
            // Иначе долгий запрос отвалится по таймауту
            update_main_window(view, app_data).await; // TODO: Можно ли опустить await?
        },
        Err(err) => {
            error!("Main window open error: {:?}", err);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

struct MainWindowView {
    view: View
}

impl MainWindowView {
    // Получаем из вьюшки имя нашего таргета
    fn get_selected_target(&self) -> Option<&str>{
        let states = self.view
            .get_info()
            .get_state();
            
        let states = match states{
            Some(states) => states,
            None => return None
        };

        states 
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
}

#[async_trait]
impl ViewActionHandler for MainWindowView {
    fn update_info(&mut self, new_info: ViewInfo){
        self.view.set_info(new_info);
    }
    fn get_view(&self) -> &View {
        &self.view
    }
    async fn on_submit(&self, trigger_id: String, app_data: Data<ApplicationData>){
        // https://api.slack.com/surfaces/modals/using#preparing_for_modals
        // Получаем из недр Json имя нужного нам таргета сборки
        let target = match self.get_selected_target(){
            Some(target) => target,
            None => {
                error!("Cannot find build target at main build window");
                return;
            }
        };

        open_build_properties_window_by_reponse(target, trigger_id, app_data).await;
    }
    async fn on_update(&self){
    }
}

////////////////////////////////////////////////////////////////////////////////

async fn update_main_window(mut view: View, app_data: Data<ApplicationData>) {
    info!("Main window view update");

    // Запрашиваем список джобов
    let jobs = match app_data.jenkins_client.request_jenkins_jobs_list().await {
        Ok(jobs) => {
            jobs
        },
        Err(err) => {
            error!("Jobs request failed: {:?}", err);
            // TODO: Save view
            return;
            // TODO: Написать ошибочное в ответ
        }
    };

    // Описываем обновление нашего окна
    // https://api.slack.com/surfaces/modals/using#interactions
    let window_view = window_json_with_jobs(Some(jobs));

    // Обновляем вьюшку
    let update_result = view
        .update_view(window_view)
        .await;
    
    match update_result {
        Ok(()) => { 
        },
        Err(err) => {
            error!("Main window update error: {:?}", err);
        }
    }

    let view_handler = Box::new(MainWindowView{
        view
    });

    // Сохраняем вьюшку для дальшнейшего использования
    app_data.push_view_handler(view_handler);
}