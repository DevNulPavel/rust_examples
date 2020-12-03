// use std::{
//     collections::{
//         HashMap
//     },
//     fmt::{
//         self
//     }
// };
use serde::{
    Deserialize
};
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
    application_data::{
        ApplicationData
    },
    active_views_holder::{
        ViewsHandlersHolder
    }
};

#[derive(Deserialize)]
pub struct SlackCommandParameters{
    pub user_id: String,
    pub user_name: String,
    pub channel_id: Option<String>,
    pub trigger_id: String,
    pub command: String,
    pub response_url: String,

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

/*impl fmt::Debug for SlackCommandParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = serde_json::to_string_pretty(self).unwrap();
        f.write_str(text.as_str())
    }
}*/

pub async fn slack_slash_command_handler(parameters: Form<SlackCommandParameters>, 
                                         app_data: Data<ApplicationData>, 
                                         views_holder: Data<ViewsHandlersHolder>) -> HttpResponse {
    //debug!("Index parameters: {:?}", parameters);

    let session = CommandSession::new(app_data, 
                                      views_holder,
                                      parameters.0.user_id,
                                      parameters.0.user_name,
                                      //Some(parameters.0.channel_id),
                                      parameters.0.trigger_id,
                                      parameters.0.response_url);

    // Открываем окно с джобами
    spawn(async move {
        // TODO: Хрень полная, но больше никак не удостовериться, что сервер слака получил ответ обработчика
        actix_web::rt::time::delay_for(std::time::Duration::from_millis(200)).await;

        open_main_build_window(session).await;
    });

    HttpResponse::Ok()
        .finish()
}
