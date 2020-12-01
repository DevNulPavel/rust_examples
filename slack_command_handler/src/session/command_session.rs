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

pub struct CommandSession{
    pub app_data: Data<ApplicationData>, 
    pub user_id: String,
    pub user_name: String,
    pub trigger_id: String, 
    pub response_url: String
}

impl CommandSession {
    pub fn new(app_data: Data<ApplicationData>,
               user_id: String,
               user_name: String,
               trigger_id: String, 
               response_url: String) -> CommandSession{
        CommandSession{
            app_data,
            user_id,
            user_name,
            trigger_id,
            response_url
        }
    }
}

impl ResponseWithError for CommandSession{
    /// Пишет сообщение об ошибке в слак + в терминал
    fn slack_response_with_error(self, error_text: String){
        //error!("{}", error_text);

        // Пишем сообщение в ответ в слак
        spawn(async move{
            let message_type = SlackMessageTaget::with_response_url(&self.response_url);
            let message_status = self.app_data
                .slack_client
                .send_message(&error_text, message_type)
                .await;

            if let Err(err) = message_status{
                error!("Slack error response failed: {:?}", err);
            }
        })
    }
}