use log::{
    debug
};
use serde::{
    Deserialize
};
use serde_json::{
    Value
};
use super::{
    request_builder::{
        SlackRequestBuilder
    },
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
    },
    search_by_name::{
        find_user_id_by_name
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
pub enum SlackImageTarget<'a>{
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

impl<'a> SlackImageTarget<'a> {
    pub fn to_user_direct(user_id: &'a str) -> SlackImageTarget{
        SlackImageTarget::User{
            user_id
        }
    }

    pub fn to_channel(channel_id: &'a str) -> SlackImageTarget<'a>{
        SlackImageTarget::Channel{
            id: channel_id
        }
    }

    pub fn to_thread(channel_id: &'a str, thread_timestamp: &'a str) -> SlackImageTarget<'a>{
        SlackImageTarget::Thread{
            id: channel_id,
            thread_ts: thread_timestamp
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////

pub struct SlackClient{
    client: SlackRequestBuilder
}

impl SlackClient {
    pub fn new(client: SlackRequestBuilder) -> SlackClient {
        SlackClient{
            client: client
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
            .build_post_request("https://slack.com/api/views.open")
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
                Ok(View::new(self.client.clone(), view))
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
            .build_post_request(url)
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
                    Ok(Some(Message::new(self.client.clone(), message, channel, ts)))
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
 
    pub async fn send_image(&self, data: Vec<u8>, commentary: String, target: SlackImageTarget<'_>) -> Result<(), SlackError> {
        // https://api.slack.com/methods/files.upload
        
        // File path
        let new_uuid = uuid::Uuid::new_v4();
        let filename = format!("{}.png", new_uuid);

        use reqwest::multipart::Part;
        use reqwest::multipart::Form;
    
        let mut form = Form::new()
            .part("initial_comment", Part::text(commentary))
            .part("filename", Part::text(filename.to_owned()))
            .part("file", Part::stream(data).file_name(filename));
    
        form = match target {
            SlackImageTarget::Channel{id} => {
                form.part("channels", Part::text(id.to_owned()))
            },
            SlackImageTarget::Thread{id, thread_ts} => {
                form
                    .part("channels", Part::text(id.to_owned()))
                    .part("thread_ts", Part::text(thread_ts.to_owned()))
            },
            SlackImageTarget::User{user_id} => {
                form.part("channels", Part::text(user_id.to_owned()))
            }
        }; 
    
        // https://api.slack.com/methods/files.upload
        #[derive(Deserialize, Debug)]
        struct File{
            id: String,
            name: String,
            title: String,
            user: String,
            channels: Vec<String>
        }
        #[derive(Deserialize, Debug)]
        #[serde(untagged)]
        enum Response{
            Ok{
                ok: bool,
                file: File
            },
            Error{
                ok: bool,
                error: String
            }
        }

        let response = self
            .client
            .build_post_request("https://slack.com/api/files.upload")
            .multipart(form)
            .send()
            .await
            .map_err(|err|{
                SlackError::RequestErr(err)
            })?
            .json::<Response>()
            .await
            .map_err(|err|{
                SlackError::JsonParseError(err)
            })?;

        match response{
            Response::Ok{ok, file} => {
                if ok {
                    debug!("File upload result: {:?}", file);
                    Ok(())
                }else{
                    Err(SlackError::Custom("File upload result: false".to_owned()))
                }
            },
            Response::Error{error, ..} => {
                Err(SlackError::Custom(error))
            }
        }
    }

    async fn find_user_id_by_email(&self, email: &str) -> Option<String> {
        // Проверяем наличие email
        if email.is_empty(){
            return None;
        }
    
        // Выполняем GET запрос
        let get_parameters = vec![
            //("token", self.token.to_owned()), // TODO: нужно протестировать
            ("email", email.to_owned())
        ];
        let response = self.client
            .build_get_request("https://slack.com/api/users.lookupByEmail")
            .query(&get_parameters)
            .send()
            .await
            .ok()?;
        //println!("{:?}", response);
    
        // Создаем структурки, в которых будут нужные значения
        #[derive(Deserialize, Debug)]
        struct UserInfo {
            id: String,
        }
        #[derive(Deserialize, Debug)]
        struct UserResponse {
            ok: bool,
            user: UserInfo,
        }
    
        // Парсим ответ в json
        let response_json = response
            .json::<UserResponse>()
            .await
            .ok()?;
        //println!("{:?}", response_json);
        
        // Результат, если все ок
        if response_json.ok {
            return Some(response_json.user.id);
        }
    
        None
    }

    // TODO: Djpdhfofnm impl Future<>
    async fn find_user_id_by_name<'a>(&'a self, user_full_name: &'a str) -> Option<String> {
        find_user_id_by_name(&self.client, user_full_name).await
    }

    pub async fn find_user_id(&self, user_email: &str, user_name: &str) -> Option<String> {
        // Ищем id сначала по email, если не вышло - по имени
    
        // Сначала ищем пользователя по email
        let id = match self.find_user_id_by_email(user_email).await {
            Some(id) => id,
            None =>{
                // Если не нашли - ищем по имени
                return self.find_user_id_by_name(user_name).await;
            }
        };
        //println!("{}", id);
        Some(id)
    }
}