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
    
}

#[derive(Deserialize, Debug)]
pub struct BuildFinishedRequest{
    build_job_url: String,

    #[serde(flatten)]
    params: BuildFinishedParameters
}

pub async fn jenkins_build_finished_handler(parameters: Json<BuildFinishedRequest>, app_data: Data<ApplicationData>) -> HttpResponse {

    if let Ok(mut awaiter) = app_data.response_awaiter.lock(){
        let BuildFinishedRequest{build_job_url, params} = parameters.0;
        awaiter.provide_build_complete_params(&build_job_url, params, Box::new(update_message_with_build_result));
    }

    HttpResponse::Ok()
        .finish()
}