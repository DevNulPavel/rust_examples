use actix_web::{
    web::{
        Data,
        Form
    },
    rt::{
        spawn
    },
    HttpResponse
};
use log::{
    debug,
    // info,
    error
};
use futures::{
    FutureExt
};
use crate::{
    jenkins::{
        api::{
            request_jenkins_jobs_list
        },
        window::{
            open_main_build_window
        },
    },
    ApplicationData
};
use super::{
    parameters::{
        SlackCommandParameters
    }
};

pub async fn jenkins_command_handler(parameters: Form<SlackCommandParameters>, app_data: Data<ApplicationData>) -> HttpResponse {
    debug!("Index parameters: {:?}", parameters);

    // Открываем окно с джобами
    open_main_build_window(app_data, &parameters.trigger_id)
        .await;

    HttpResponse::Ok()
        .finish()
}
