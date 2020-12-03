use serde::{
    Deserialize
};
use actix_web::{
    web::{
        Data,
        Json
    },
    rt::{
        spawn
    },
    HttpResponse
};
// use log::{
//     debug,
//     info,
//     error
// };
use crate::{
    ApplicationData,
    response_awaiter_holder::{
        ResponseAwaiterHolder
    }
};
use super::{
    event_processing::{
        process_jenkins_event
    },
    message_event::{
        MessageEvent
    }
};


/*#[derive(Deserialize, Debug)]
pub struct AuthInformation {
    enterprise_id: String,
    team_id: String,
    user_id: String,
    is_bot: bool
}*/

// https://api.slack.com/events-api
// https://api.slack.com/apps/A01BSSSHB36/event-subscriptions?
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum SlackEventParameters{
    /// Подтверждение урла
    /// https://api.slack.com/events/url_verification
    #[serde(rename = "url_verification")]
    Verify{
        token: String,
        challenge: String,
    },
    
    /// Сообщение для бота
    /// https://api.slack.com/events-api#the-events-api__receiving-events__events-dispatched-as-json
    #[serde(rename = "event_callback")]
    Message{
        event: MessageEvent,
        //authed_users: Option<Vec<String>>,
        //authorizations: Option<AuthInformation>
    }
}

/*impl fmt::Debug for SlackCommandParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = serde_json::to_string_pretty(self).unwrap();
        f.write_str(text.as_str())
    }
}*/


pub async fn slack_events_handler(parameters: Json<SlackEventParameters>, app_data: Data<ApplicationData>, awaiter: Data<ResponseAwaiterHolder>) -> HttpResponse {
    // Про ответы на события: https://api.slack.com/events-api#the-events-api__responding-to-events
    match parameters.into_inner() {
        SlackEventParameters::Verify{challenge, ..} => {
            return HttpResponse::Ok().body(challenge);
        }

        SlackEventParameters::Message{event} => {
            // Запускаем асинхронную задачу в работу, чтобы моментально ответить на запрос
            spawn(async move{
                // TODO: Хрень полная, но больше никак не удостовериться, что сервер слака получил ответ обработчика
                actix_web::rt::time::delay_for(std::time::Duration::from_millis(200)).await;

                process_jenkins_event(event, app_data, awaiter).await;
            })
        }
    }

    HttpResponse::Ok().finish()
}