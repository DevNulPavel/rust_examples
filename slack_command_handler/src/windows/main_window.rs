use log::{
    info,
    debug,
    error
};
// use async_trait::{
//     async_trait
// };
use actix_web::{ 
    rt::{
        spawn
    }
};
use serde_json::{
    Value
};
use slack_client_lib::{
    View,
    ViewInfo
};
use crate::{
    session::{
        CommandSession,
        WindowSession
    },
    jenkins::{
        // JenkinsClient,
        JenkinsTarget
    },
    slack_response_with_error
};
use super::{
    view_action_handler::{
        ViewActionHandler
    },
    properties_window::{
        open_build_properties_window_by_reponse
    }
};


const MAIN_WINDOW_ID: &str = "MAIN_WINDOW_ID";

fn window_json_with_jobs(jobs: Option<&Vec<JenkinsTarget>>) -> Value {
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
pub async fn open_main_build_window(session: CommandSession) {
    // Выполняем наш запрос
    // TODO: Вернуть ошибку
    //info!("Open main window with: {:?}", session);

    let window_view = window_json_with_jobs(None);

    let window = serde_json::json!({
        "trigger_id": session.trigger_id,
        "view": window_view
    });

    let open_result = session
        .app_data
        .slack_client
        .open_view(window)
        .await;
    
    match open_result {
        Ok(view) => {
            let session = WindowSession::new(session.app_data, 
                                             session.views_holder,
                                             session.user_id, 
                                             session.user_name, 
                                             session.trigger_id);

            // Запускаем асинхронный запрос, чтобы моментально ответить
            // Иначе долгий запрос отвалится по таймауту
            update_main_window(view, session).await; // TODO: Можно ли опустить await?
        },
        Err(err) => {
            // Пишем сообщение в ответ в слак
            slack_response_with_error!(session, format!("Main window open error: {:?}", err));
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

struct MainWindowView {
    view: View,
    jobs: Vec<JenkinsTarget>
}

impl MainWindowView {
    // Получаем из вьюшки имя нашего таргета
    fn get_selected_target<'a>(&'a self) -> Option<&'a str>{
        let states = self.view
            .get_info()
            .get_state();
            
        let states = match states{
            Some(states) => states,
            None => {
                error!("Empty states");
                return None;
            }
        };

        debug!("States: {:?}", states);

        states 
            .get("values")
            .and_then(|val|{
                val.get("build_target_block_id")
            })
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

// #[async_trait]
impl ViewActionHandler for MainWindowView {
    fn update_info(&mut self, new_info: ViewInfo){
        self.view.set_info(new_info);
    }
    fn get_view(&self) -> &View {
        &self.view
    }
    fn on_submit(self: Box<Self>, session: WindowSession){
        // https://api.slack.com/surfaces/modals/using#preparing_for_modals
        // Получаем из недр Json имя нужного нам таргета сборки
        let target = match self.as_ref().get_selected_target(){
            Some(target) => target.to_owned(),
            None => {
                // Пишем сообщение в ответ в слак
                slack_response_with_error!(session, "Cannot find build target at main build window".to_owned());
                return;
            }
        };

        // TODO: Более оптимальный поиск??
        let found_job = self
            .jobs
            .into_iter()
            .find(|job|{
                job.get_info().name == target
            });

        // Разворачиваем результат
        let found_job = match found_job {
            Some(job) => job,
            None => {
                // Пишем сообщение в ответ в слак
                slack_response_with_error!(session, format!("Cannot find job object with name {}", target));
                return;
            }
        };

        spawn(async move {
            open_build_properties_window_by_reponse(found_job, session).await;
        });
    }
    fn on_update(&self){
    }
    fn on_close(self: Box<Self>, _: WindowSession){
    }
}

////////////////////////////////////////////////////////////////////////////////

async fn update_main_window(mut view: View, session: WindowSession) {
    info!("Main window view update");

    // Запрашиваем список джобов
    let jobs = match session.app_data.jenkins_client.request_jenkins_targets_list().await {
        Ok(jobs) => {
            jobs
        },
        Err(err) => {
            // Пишем сообщение в ответ в слак
            slack_response_with_error!(session, format!("Jobs request failed: {:?}", err));
            return;
            // TODO: Написать ошибочное в ответ
        }
    };

    // Описываем обновление нашего окна
    // https://api.slack.com/surfaces/modals/using#interactions
    let window_view = window_json_with_jobs(Some(&jobs));

    // Обновляем вьюшку
    let update_result = view
        .update_view(window_view)
        .await;
    
    match update_result {
        Ok(()) => { 
            let view_handler = Box::new(MainWindowView{
                jobs,
                view
            });
        
            // Сохраняем вьюшку для дальшнейшего использования
            session.views_holder.push_view_handler(view_handler);
        },
        Err(err) => {
            // Пишем сообщение в ответ в слак
            slack_response_with_error!(session, format!("Main window update error: {:?}", err));
        }
    }
}