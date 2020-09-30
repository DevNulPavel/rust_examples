use std::{
    fmt,
    collections::{
        HashMap
    }
};
use serde::{
    Serialize,
    Deserialize
};
use serde_json::{
    Value
};

// {
//     "type": "modal",
//     "callback_id": "modal-identifier",
//     "title": {
//       "type": "plain_text",
//       "text": "Just a modal"
//     },
//     "blocks": [
//       {
//         "type": "section",
//         "block_id": "section-identifier",
//         "text": {
//           "type": "mrkdwn",
//           "text": "*Welcome* to ~my~ Block Kit _modal_!"
//         },
//         "accessory": {
//           "type": "button",
//           "text": {
//             "type": "plain_text",
//             "text": "Just a button",
//           },
//           "action_id": "button-identifier",
//         }
//       }
//     ],
//   }

#[derive(Serialize)]
pub struct SlackWindowTitle{
    #[serde(rename = "type")]
    pub title_type: &'static str,
    pub text: String,
}

#[derive(Serialize)]
pub struct SlackWindowBlock{
    #[serde(rename = "type")]
    pub block_type: &'static str
}

#[derive(Serialize)]
pub struct SlackWindow{
    #[serde(rename = "type")]
    pub window_type: &'static str,
    pub callback_id: String,
    pub title: SlackWindowTitle,
    pub blocks: Vec<SlackWindowBlock>
}

impl SlackWindow {
    pub fn new() -> SlackWindow{
        SlackWindow{
            window_type: "modal",
            callback_id: String::from(""),
            title: SlackWindowTitle{
                title_type: "plain_text",
                text: String::from("Test window")
            },
            blocks: vec![
                SlackWindowBlock{
                    block_type: "plain_text"
                }
            ]
        }
    }
}

// https://api.slack.com/reference/interaction-payloads/block-actions

#[derive(Deserialize, Serialize, Debug)]
pub struct SlackWindowParametersViewInfo{
    pub id: String,

    // Прочие поля
    #[serde(flatten)]
    other: HashMap<String, Value>
}


#[derive(Deserialize, Serialize)]
pub struct SlackWindowParametersPayload{
    #[serde(rename = "type")]
    pub payload_type: String,
    pub trigger_id: String,
    pub actions: Vec<Value>,
    pub view: SlackWindowParametersViewInfo,

    // pub user: HashMap<String, serde_json::Value>,
    // pub view: HashMap<String, Value>,

    // Прочие поля
    // #[serde(flatten)]
    // other: HashMap<String, Value>
}

impl fmt::Debug for SlackWindowParametersPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(serde_json::to_string_pretty(self)
            .unwrap()
            .as_str())
    }
}

#[derive(Deserialize, Debug)]
pub struct SlackWindowParameters{
    pub payload: String
}