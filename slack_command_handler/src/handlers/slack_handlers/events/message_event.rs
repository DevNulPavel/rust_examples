use serde::{
    Deserialize
};

// https://api.slack.com/events-api#event_type_structure
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum MessageEvent{
    /// Упоминание бота в канале
    /// https://api.slack.com/events/app_mention
    #[serde(rename = "app_mention")]
    AppMention{
        user: String,
        channel: String,
        text: String,
        ts: String,
    },
    
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