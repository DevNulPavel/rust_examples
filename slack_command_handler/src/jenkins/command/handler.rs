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
    spawn(async move {
        // TODO: Хрень полная, но больше никак не удостовериться, что сервер слака получил ответ обработчика
        actix_web::rt::time::delay_for(std::time::Duration::from_millis(200)).await;

        open_main_build_window(app_data, parameters.0.trigger_id).await;
    });

    HttpResponse::Ok()
        .finish()
}
