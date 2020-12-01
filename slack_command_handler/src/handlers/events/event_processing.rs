use actix_web::{
    web::{
        Data,
    },
};
use log::{
    debug,
    // info,
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
    let jobs = session
        .app_data
        .jenkins_client
        .request_jenkins_jobs_list()
        .await;
    
    let jobs = unwrap_error_with_slack_response_and_return!(jobs, session, "Jenkins' jobs request failed: {:?}");

    let found_job = jobs
        .iter()
        .find(|job|{
            job.get_info().name == target
        });

    let found_job = unwrap_option_with_slack_response_and_return!(found_job, session, "Required job is not found");

    // TODO: ???
    let found_parameters = found_job
        .request_jenkins_job_info()
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

    let job_start_result = found_job
        .start_job(parameters)
        .await;

    let url = unwrap_error_with_slack_response_and_return!(job_start_result, 
                                                                        session, 
                                                                        "Jenkins job start error: {:?}");

    
    // Тестовое сообщение
    let test_message = format!("```Target: {}\nBranch: {}\nUrl: {}```", target, branch, url);
    session
        .app_data
        .slack_client
        .send_message(&test_message, SlackMessageTaget::to_channel(&session.channel_id))
        .await
        .ok();
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
            
            //let session = EventSession::new(app_data, user, channel);
        }
    }
}