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

    #[allow(dead_code)]
    pub fn to_channel_ephemeral(channel_id: &'a str, user_id: &'a str) -> SlackMessageTaget<'a>{
        SlackMessageTaget::Ephemeral{
            channel_id,
            user_id
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
                    json["user"] = serde_json::Value::from(user_id)
                },
                SlackMessageTaget::Channel{id} => {
                    json["channel"] = serde_json::Value::from(id)
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
            SlackMessageTaget::Channel{..} | SlackMessageTaget::User{..} => "https://slack.com/api/chat.postMessage"
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
    
}