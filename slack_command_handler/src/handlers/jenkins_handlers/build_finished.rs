use std::{
    sync::{
        Mutex
    }
};
use log::{
    debug
};
use serde::{
    Deserialize
};
use actix_web::{
    web::{
        Data,
        Form,
        Json
    },
    rt::{
        spawn
    },
    HttpResponse
};
// use log::{
    // debug,
    // info,
    //error
// };
use crate::{
    windows::{
        open_main_build_window
    },
    session::{
        CommandSession
    },
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
    build_number: String,
    git_commit: Option<String>,
    git_branch: Option<String>,
    build_file_link: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct BuildFinishedRequest{
    build_job_url: String,

    #[serde(flatten)]
    params: BuildFinishedParameters
}

pub async fn jenkins_build_finished_handler(parameters: Form<BuildFinishedRequest>, app_data: Data<ApplicationData>, awaiter: Data<Mutex<ResponseAwaiterHolder>>) -> HttpResponse {
    debug!("Jenkins build finished params: {:?}", parameters.0);

    if let Ok(mut awaiter) = awaiter.lock(){
        let BuildFinishedRequest{build_job_url, params} = parameters.0;
        awaiter.provide_build_complete_params(&build_job_url, params, app_data, Box::new(update_message_with_build_result));
    }

    HttpResponse::Ok()
        .finish()
}