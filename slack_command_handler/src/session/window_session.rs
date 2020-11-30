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
    base_session::{
        BaseSession
    },
    error_response_trait::{
        ResponseWithError
    }
};

pub struct WindowSession{
    pub base: BaseSession
}

impl WindowSession {
    pub fn new(app_data: Data<ApplicationData>,
               user_id: String,
               user_name: String,
               trigger_id: String) -> WindowSession{
        WindowSession{
            base: BaseSession::new(app_data, user_id, user_name, trigger_id)
        }
    }

    pub fn new_with_base(base: BaseSession) -> WindowSession{
        WindowSession{
            base
        }
    }
}

impl ResponseWithError for WindowSession{
    /// Пишет сообщение об ошибке в слак + в терминал
    fn slack_response_with_error(self, error_text: String){
        //error!("{}", error_text);

        // Пишем сообщение в ответ в слак
        spawn(async move{
            let formatted_text = format!("*Jenkins bot error:*```{}```", error_text);

            let message_type = SlackMessageTaget::to_user_direct(&self.base.user_id);
            let message_status = self.base.app_data
                .slack_client
                .send_message(&formatted_text, message_type)
                .await;

            if let Err(err) = message_status{
                error!("Slack error response failed: {:?}", err);
            }
        })
    }
}