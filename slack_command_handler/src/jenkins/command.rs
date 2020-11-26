use std::{
    fmt
};
use actix_web::{ 
    web,
    Responder,
    HttpResponse
};
use serde::{
    Serialize,
    Deserialize
};
use log::{
    debug,
    // info,
    error
};
use futures::{
    FutureExt
};
use super::{
    api::{
        request_jenkins_jobs_list
    },
    window::{
        open_main_build_window
    }
};
use crate::{
    ApplicationData
};


#[derive(Deserialize, Serialize)]
pub struct SlackCommandParameters{
    user_id: String,
    user_name: String,
    trigger_id: String,
    command: String,

    // pub token: String,
    // pub text: String,
    // channel_id: String,
    // team_id: String,
    // team_domain: String,
    // enterprise_id: String,
    // enterprise_name: String,
    // channel_name: String,
    // response_url: String,
    // api_app_id: String

    // Так можно получить прочие необязательные параметры
    // https://serde.rs/attr-flatten.html
    // #[serde(flatten)]
    // extra: HashMap<String, Value>
}

impl fmt::Debug for SlackCommandParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = serde_json::to_string_pretty(self).unwrap();
        f.write_str(text.as_str())
    }
}

pub async fn jenkins_command_handler(parameters: web::Form<SlackCommandParameters>, app_data: web::Data<ApplicationData>) -> HttpResponse {
    debug!("Index parameters: {:?}", parameters);

    // Запрашиваем список джобов
    request_jenkins_jobs_list(&app_data.http_client, &app_data.jenkins_auth)
        .then(|res| async {
            match res {
                Ok(jobs) => {
                    // Открываем окно с джобами
                    open_main_build_window(&app_data, &parameters.trigger_id, jobs)
                        .await
                },
                Err(err) => {
                    error!("Jobs request failed: {:?}", err);
                    HttpResponse::Ok()
                        .body(format!("{:?}", err))
                }
            }
        })
        .await
}
