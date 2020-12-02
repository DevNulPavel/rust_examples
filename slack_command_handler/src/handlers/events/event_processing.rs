use actix_web::{
    web::{
        Data,
    },
};
use log::{
    debug,
    info,
    error
};
use crate::{
    session::{
        EventSession
    },
    slack::{
        SlackMessageTaget
    },
    jenkins::{
        Parameter
    },
    ApplicationData,
    slack_response_with_error,
    unwrap_error_with_slack_response_and_return,
    unwrap_option_with_slack_response_and_return
};
use super::{
    message_event::{
        MessageEvent
    },
    text_parser::{
        parse_mention_params
    }
};


async fn start_jenkins_job(target: &str, branch: &str, session: EventSession) {
    let targets = session
        .app_data
        .jenkins_client
        .request_jenkins_targets_list()
        .await;
    
    let targets = unwrap_error_with_slack_response_and_return!(targets, session, "Jenkins' jobs request failed: {:?}");

    let found_target = targets
        .iter()
        .find(|target_obj|{
            target_obj.get_info().name == target
        });

    let found_target = unwrap_option_with_slack_response_and_return!(found_target, session, "Required job is not found");

    // TODO: ???
    let found_parameters = found_target
        .request_jenkins_target_info()
        .await;

    let found_parameters = unwrap_error_with_slack_response_and_return!(found_parameters, 
                                                                        session, 
                                                                        "Jenkins job's parameters request failed: {:?}");

    let branch_param = found_parameters
        .into_iter()
        .find(|param|{
            match param {
                Parameter::String{name, ..} => {
                    name == "BRANCH"
                },
                Parameter::Git{name, ..} => {
                    name == "BRANCH"
                },
                _ => {
                    false
                }
            }
        });

    unwrap_option_with_slack_response_and_return!(branch_param, 
                                                  session, 
                                                  "Branch param is not found in this target");

    let parameters = serde_json::json!({
        "BRANCH": branch.to_owned()
    });

    let job_start_result = found_target
        .start_job(parameters)
        .await;

    let mut job = unwrap_error_with_slack_response_and_return!(job_start_result, 
                                                           session, 
                                                           "Jenkins job start error: {:?}");

    
    // Тестовое сообщение
    let test_message = format!(":zhdun:```Target: {}\nBranch: {}\nTarget: {}```", target, branch, found_target.get_info().url);
    
    // Шлем сообщение
    let message = session
        .app_data
        .slack_client
        .send_message(&test_message, SlackMessageTaget::to_channel(&session.channel_id))
        .await;

    let message = unwrap_error_with_slack_response_and_return!(message, 
                                                               session, 
                                                               "Message send failed: {:?}");

    // Можем ли мы вообще модифицировать сообщение?
    let mut message = match message{
        Some(message) => message,
        None => return
    };

    // Можно запустить пулинг для ожидания финальной ссылки, затем обновить сообщение
    // Ограничить продолжительность пулинга статуса 30 минутами
    info!("Job url pooling started for url: {}", job.get_info_api_url());
    use std::time::{
        Instant,
        Duration
    };
    let complete_time = Instant::now()
        .checked_add(Duration::from_secs(60 * 30))
        .expect("Complete time create failed");
    while complete_time > std::time::Instant::now() {

        actix_web::rt::time::delay_for(std::time::Duration::from_secs(10)).await;

        let result = job.try_to_get_real_job_url().await;
        let result = unwrap_error_with_slack_response_and_return!(result, session, "Real job url request failed: {:?}");

        match result {
            Some(real_url) => {
                // Обновляем сообщение
                let new_text = format!(":jenkins:```Target: {}\nBranch: {}\nJob url: {}```", target, branch, real_url);
                let update_result = message.update_text(&new_text).await;
                unwrap_error_with_slack_response_and_return!(update_result, session, "Message update failed: {:?}");

                break;
            },
            None =>{
            }
        }
    }
}


pub async fn process_jenkins_event(event: MessageEvent, app_data: Data<ApplicationData>)  {
    match event {
        MessageEvent::AppMention{channel, text, user} => {
            debug!("Channel message event: channel {}, text {}, user {}", channel, text, user);

            // Создание сессии
            let session = EventSession::new(app_data, user, channel);

            // Парсинг параметров билда
            let params = match parse_mention_params(&text){
                Some(params) => params,
                None => {
                    slack_response_with_error!(
                        session, 
                        "Supported build request format: @<bot_name> <jenkins_target> <branch>".to_owned()
                    );
                    return;
                }
            };

            // Пытаемся стартануть билд
            start_jenkins_job(params.target_name, params.branch_name, session).await;
        },
        MessageEvent::DirectMessage{channel, text, user} => {
            debug!("Channel message event: channel {}, text {}, user {}", channel, text, user);
            
            // TODO: С помощью сообщений в личку заниматься управлением бота?
        }
    }
}