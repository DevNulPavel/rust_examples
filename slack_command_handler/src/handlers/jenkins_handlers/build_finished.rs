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
    helpers::{
        send_message_with_build_result_into_thread,
        send_message_with_build_result
    },
    ApplicationData
};

#[derive(Deserialize, Debug)]
pub struct BuildResultFileInfo{
    pub build_file_link: String,
    pub build_file_commentary: String
}

#[derive(Deserialize, Debug)]
pub struct BuildResultUserInfo{
    pub build_user_id: String,
    pub build_user_name: String,
    pub build_user_email: String,
}

#[derive(Deserialize, Debug)]
pub struct BuildResultJobInfo{
    pub build_job_url: String,
    pub build_number: String,
}

#[derive(Deserialize, Debug)]
pub struct BuildResultGitInfo{
    pub git_commit: String,
    pub git_branch: String,
}

#[derive(Deserialize, Debug)]
pub struct BuildFinishedParameters{
    #[serde(flatten)]
    pub job_info: BuildResultJobInfo,

    #[serde(flatten)]
    pub git_info: Option<BuildResultGitInfo>,

    #[serde(flatten)]
    pub user_info: Option<BuildResultUserInfo>,

    #[serde(flatten)]
    pub file_info: Option<BuildResultFileInfo>,

    pub default_channel: Option<String>,
}

pub async fn jenkins_build_finished_handler(parameters: Form<BuildFinishedParameters>, app_data: Data<ApplicationData>, awaiter: Data<ResponseAwaiterHolder>) -> HttpResponse {
    debug!("Jenkins build finished params: {:?}", parameters.0);

    // Если у нас есть id пользователя, то мы проверяем кто начал сборку
    // Если сборка была начата не ботом, то просто пишем сообщение в личку
    // Если сборка была начата ботом - тогда обрабатываем
    if let Some(ref user_info) = parameters.0.user_info {
        if user_info.build_user_id == app_data.jenkins_client.get_jenkins_user(){
            debug!("Thread response await: {:?}", parameters.0);
            // Начинаем ждать
            let url = parameters.job_info.build_job_url.clone();
            awaiter.provide_build_complete_params(url, parameters.0, app_data, send_message_with_build_result_into_thread);
        }else {
            debug!("Direct message send: {:?}", parameters.0);
            send_message_with_build_result(parameters.0, app_data);
        }
    }else{
        // По-умолчанию считаем, что сборка была создана ботом
        let url = parameters.job_info.build_job_url.clone();
        awaiter.provide_build_complete_params(url, parameters.0, app_data, send_message_with_build_result_into_thread);    
    }

    HttpResponse::Ok()
        .finish()
}