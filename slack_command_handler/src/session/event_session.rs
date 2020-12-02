use log::{
    // info,
    // debug,
    error
};
use actix_web::{ 
    rt::{
        spawn
    },
    web::{
        Data
    }
};
use crate::{
    application_data::{
        ApplicationData
    },
    slack::{
        SlackMessageTaget
    }
};
use super::{
    error_response_trait::{
        ResponseWithError
    }
};

pub struct EventSession{
    pub app_data: Data<ApplicationData>,
    pub user_id: String,
    pub channel_id: String,
    pub trigget_message_ts: String,
}

impl EventSession {
    pub fn new(app_data: Data<ApplicationData>,
               user_id: String,
               channel_id: String,
               trigget_message_ts: String) -> EventSession{
        EventSession{
            app_data,
            user_id,
            channel_id,
            trigget_message_ts
        }
    }
}

impl ResponseWithError for EventSession{
    /// Пишет сообщение об ошибке в слак + в терминал
    fn slack_response_with_error(self, error_text: String){
        //error!("{}", error_text);

        // Пишем сообщение в ответ в слак
        spawn(async move{
            let formatted_text = format!("```{}```", error_text);

            //let message_type = SlackMessageTaget::to_channel_ephemeral(&self.channel_id, &self.user_id);
            let message_type = SlackMessageTaget::to_channel(&self.channel_id);

            let message_status = self.app_data
                .slack_client
                .send_message(&formatted_text, message_type)
                .await;

            if let Err(err) = message_status{
                error!("Slack error response failed: {:?}", err);
            }
        })
    }
}