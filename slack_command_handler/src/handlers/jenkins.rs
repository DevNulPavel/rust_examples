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
    ApplicationData
};


#[derive(Deserialize)]
pub struct SlackCommandParameters{
    pub user_id: String,
    pub user_name: String,
    pub trigger_id: String,
    pub command: String,

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

pub async fn jenkins_command_handler(parameters: Form<SlackCommandParameters>, app_data: Data<ApplicationData>) -> HttpResponse {
    //debug!("Index parameters: {:?}", parameters);

    // Открываем окно с джобами
    spawn(async move {
        // TODO: Хрень полная, но больше никак не удостовериться, что сервер слака получил ответ обработчика
        actix_web::rt::time::delay_for(std::time::Duration::from_millis(200)).await;

        open_main_build_window(app_data, parameters.0.trigger_id).await;
    });

    HttpResponse::Ok()
        .finish()
}



#[cfg(test)]
mod tests {
    use super::{
        *
    };

    #[actix_rt::test]
    async fn test_jenkins_command_handler() {
        // TODO: Fake request https://actix.rs/docs/testing/
    }
}