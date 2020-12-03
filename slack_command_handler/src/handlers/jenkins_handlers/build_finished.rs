use log::{
    debug
};
use serde::{
    Deserialize
};
use actix_web::{
    web::{
        Data,
        Form
    },
    HttpResponse
};
// use log::{
    // debug,
    // info,
    //error
// };
use crate::{
    response_awaiter_holder::{
        ResponseAwaiterHolder
    },
    handlers::{
        slack_handlers::{
            update_message_with_build_result
        }
    },
    ApplicationData
};

#[derive(Deserialize, Debug)]
pub struct BuildFinishedParameters{
    build_job_url: String,
    build_number: String,
    build_user_id: Option<String>,
    build_user: Option<String>,
    git_commit: Option<String>,
    git_branch: Option<String>,
    build_file_link: Option<String>
}

pub async fn jenkins_build_finished_handler(parameters: Form<BuildFinishedParameters>, app_data: Data<ApplicationData>, awaiter: Data<ResponseAwaiterHolder>) -> HttpResponse {
    debug!("Jenkins build finished params: {:?}", parameters.0);

    // Если у нас есть id пользователя, то мы проверяем кто начал сборку
    // Если сборка была начата не ботом, то просто пишем сообщение в личку
    // Если сборка была начата ботом - тогда обрабатываем
    if let Some(ref user_id) = parameters.0.build_user_id {
        if user_id == app_data.jenkins_client.get_jenkins_user(){
            // Начинаем ждать
            let url = parameters.build_job_url.clone();
            awaiter.provide_build_complete_params(url, parameters.0, app_data, Box::new(update_message_with_build_result));    
        }else{

        }
    }else{
        // По-умолчанию считаем, что сборка была создана ботом
        let url = parameters.build_job_url.clone();
        awaiter.provide_build_complete_params(url, parameters.0, app_data, Box::new(update_message_with_build_result));    
    }

    HttpResponse::Ok()
        .finish()
}