use serde::{
    Deserialize
};

#[derive(Deserialize, Debug)]
pub struct AppMentionMessageInfo{
    pub user: String,
    pub channel: String,
    pub text: String,
    pub ts: String,
}

// https://api.slack.com/events-api#event_type_structure
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum MessageEvent{
    /// Упоминание бота в канале
    /// https://api.slack.com/events/app_mention
    #[serde(rename = "app_mention")]
    AppMention(AppMentionMessageInfo),
    
    /// Сообщение в личку
    /// https://api.slack.com/events/message.im
    #[serde(rename = "message")]
    DirectMessage{
        user: String,
        channel: String,
        text: String,
        //"channel_type": "im"
    }
}