// use log::{
    // error
// };
use serde::{
    Deserialize
};
use serde_json::{
    Value
};
use reqwest::{
    Client
};
use super::{
    error::{
        SlackError,
        ViewOpenErrorInfo
    },
    // view_open_response::{
        // ViewOpenResponse,
        // ViewUpdateResponse,
        // ViewInfo
    // },
    message::{
        MessageInfo,
        Message
    },
    view::{
        ViewInfo,
        View
    }
};

//////////////////////////////////////////////////////////////////////////////////////////

/// Таргет для сообщения в личку
#[allow(dead_code)]
pub enum SlackMessageTaget<'a>{
    /// Сообщение в канал, которое видно всем
    Channel{
        id: &'a str
    },
    /// Сообщение в тред
    Thread{
        id: &'a str,
        thread_ts: &'a str
    },
    /// Сообщение в личку
    User{
        user_id: &'a str
    },
    /// Сообщение в канал, но видное только конкретному пользователю
    Ephemeral{
        channel_id: &'a str,
        user_id: &'a str
    },
    /// Сообщение в ответ на какое-то взаимодействие
    ResponseUrl{
        url: &'a str
    }
}

impl<'a> SlackMessageTaget<'a> {
    pub fn with_response_url(url: &'a str) -> SlackMessageTaget{
        SlackMessageTaget::ResponseUrl{
            url
        }
    }

    pub fn to_user_direct(user_id: &'a str) -> SlackMessageTaget{
        SlackMessageTaget::User{
            user_id
        }
    }

    pub fn to_channel(channel_id: &'a str) -> SlackMessageTaget<'a>{
        SlackMessageTaget::Channel{
            id: channel_id
        }
    }

    pub fn to_thread(channel_id: &'a str, thread_timestamp: &'a str) -> SlackMessageTaget<'a>{
        SlackMessageTaget::Thread{
            id: channel_id,
            thread_ts: thread_timestamp
        }
    }

    #[allow(dead_code)]
    pub fn to_channel_ephemeral(channel_id: &'a str, user_id: &'a str) -> SlackMessageTaget<'a>{
        SlackMessageTaget::Ephemeral{
            channel_id,
            user_id
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////

/// Таргет для сообщения в личку
#[allow(dead_code)]
pub enum SlackImageTaget<'a>{
    /// Сообщение в канал, которое видно всем
    Channel{
        id: &'a str
    },
    /// Сообщение в тред
    Thread{
        id: &'a str,
        thread_ts: &'a str
    },
    /// Сообщение в личку
    User{
        user_id: &'a str
    }
}

impl<'a> SlackImageTaget<'a> {
    pub fn to_user_direct(user_id: &'a str) -> SlackImageTaget{
        SlackImageTaget::User{
            user_id
        }
    }

    pub fn to_channel(channel_id: &'a str) -> SlackImageTaget<'a>{
        SlackImageTaget::Channel{
            id: channel_id
        }
    }

    pub fn to_thread(channel_id: &'a str, thread_timestamp: &'a str) -> SlackImageTaget<'a>{
        SlackImageTaget::Thread{
            id: channel_id,
            thread_ts: thread_timestamp
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////

pub struct SlackClient{
    client: Client,
    token: String
}

impl SlackClient {
    pub fn new(client: Client, token: &str) -> SlackClient {
        SlackClient{
            client: client,
            token: token.to_owned()
        }
    }

    pub async fn open_view(&self, window_json: Value) -> Result<View, SlackError>{
        // https://serde.rs/enum-representations.html
        // https://api.slack.com/methods/views.open#response
        #[derive(Deserialize, Debug)]
        #[serde(untagged)]
        pub enum ViewOpenResponse{
            Ok{ view: ViewInfo },
            Error(ViewOpenErrorInfo)
        }

        let response = self.client
            .post("https://slack.com/api/views.open")
            .bearer_auth(&self.token)
            .header("Content-type", "application/json")
            .body(serde_json::to_string(&window_json).unwrap())
            .send()
            .await
            .map_err(|err|{
                SlackError::RequestErr(err)
            })?
            .json::<ViewOpenResponse>()
            .await
            .map_err(|err|{
                SlackError::JsonParseError(err)
            })?;

        match response {
            ViewOpenResponse::Ok{view} => {
                Ok(View::new(self.client.clone(), self.token.clone(), view))
            },
            ViewOpenResponse::Error(err) => {
                Err(SlackError::ViewOpenError(err))
            }
        }
    }

    pub async fn send_message(&self, message: &str, target: SlackMessageTaget<'_>) -> Result<Option<Message>, SlackError> {
        // https://api.slack.com/messaging/sending
        // https://api.slack.com/methods/chat.postMessage

        // Наше сообщения
        let message_json = {
            let mut json = serde_json::json!({
                "text": message
            });
            // TODO: Может как-то оптимальнее добавлять канал??
            match target{
                SlackMessageTaget::ResponseUrl{..} => {
                },
                SlackMessageTaget::Ephemeral{channel_id, user_id} => {
                    json["channel"] = serde_json::Value::from(channel_id);
                    json["user"] = serde_json::Value::from(user_id);
                },
                SlackMessageTaget::Channel{id} => {
                    json["channel"] = serde_json::Value::from(id);
                },
                SlackMessageTaget::Thread{id, thread_ts} => {
                    json["channel"] = serde_json::Value::from(id);
                    json["thread_ts"] = serde_json::Value::from(thread_ts);
                },
                SlackMessageTaget::User{user_id} => {
                    json["channel"] = serde_json::Value::from(user_id);
                    json["as_user"] = serde_json::Value::from(true);
                }
            };
            json
        };

        // Ответ на постинг сообщения
        #[derive(Deserialize, Debug)]
        #[serde(untagged)]
        enum MessageResponse{
            Ok{
                ok: bool,
                channel: String,
                ts: String,
                message : MessageInfo
            },
            OtherOk{
                ok: bool
            },
            Err{
                ok: bool,
                error: String
            }
        };

        // Либо можем использовать стандартный урл, 
        // либо можем использовать урл для отправки сообщения
        // https://api.slack.com/messaging/sending#sending_methods
        // https://api.slack.com/interactivity/handling#message_responses
        let url = match target{
            SlackMessageTaget::ResponseUrl{url} => url,
            SlackMessageTaget::Ephemeral{..} => "https://slack.com/api/chat.postEphemeral",
            SlackMessageTaget::Channel{..} | 
                SlackMessageTaget::User{..} |
                SlackMessageTaget::Thread{..} => "https://slack.com/api/chat.postMessage"
        };

        let response = self.client
            .post(url)
            .bearer_auth(&self.token)
            .header("Content-type", "application/json")
            .body(serde_json::to_string(&message_json).unwrap())
            .send()
            .await
            .map_err(|err|{
                SlackError::RequestErr(err)
            })?;

        let response = response
            .json::<MessageResponse>()
            .await
            .map_err(|err|{
                SlackError::JsonParseError(err)
            })?;

        match response {
            MessageResponse::Ok{ok, channel, ts, message} =>{
                if ok {
                    Ok(Some(Message::new(self.client.clone(), self.token.clone(), message, channel, ts)))
                }else{
                    return Err(SlackError::Custom(format!("Slack response: {}", ok)))
                }
            },
            MessageResponse::OtherOk{ok} =>{
                if ok {
                    Ok(None)
                }else{
                    return Err(SlackError::Custom(format!("Slack response: {}", ok)))
                }
            },
            MessageResponse::Err{error, ..} => {
                return Err(SlackError::Custom(error))
            }
        }
    }
 
    pub async fn send_image(&self, data: Vec<u8>, text: &str, commentary: &str, target: SlackImageTaget<'_>) -> Result<(), SlackError> {
        // https://api.slack.com/methods/files.upload
        
        // File path
        let new_uuid = uuid::Uuid::new_v4();
        let filename = format!("{}.png", new_uuid);
    
        // Есть или нет комментарий?
        let commentary = match commentary.len() {
            0 => commentary,
            _ => text
        };

        use reqwest::multipart::Part;
        use reqwest::multipart::Form;
    
        let mut form = Form::new()
            .part("initial_comment", Part::text(commentary.to_owned()))
            .part("filename", Part::text(filename.to_owned()))
            .part("file", Part::stream(data).file_name(filename.to_owned()));
    
        form = match target {
            SlackImageTaget::Channel{id} => {
                form.part("channels", Part::text(id.to_owned()))
            },
            SlackImageTaget::Thread{id, thread_ts} => {
                form
                    .part("channels", Part::text(id.to_owned()))
                    .part("thread_ts", Part::text(thread_ts.to_owned()))
            },
            SlackImageTaget::User{..} => {
                // TODO: Открытие канала лички
                form
            }
        }; 
    
        self
            .client
            .post("https://slack.com/api/files.upload")
            .bearer_auth(&self.token)
            .multipart(form)
            .send()
            .await
            .map_err(|err|{
                SlackError::RequestErr(err)
            })?;

        Ok(())
    }
}