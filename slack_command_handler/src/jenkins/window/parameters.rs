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

#[derive(Deserialize, Serialize, Debug)]
pub struct WindowState{
    pub values: HashMap<String, Value>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WindowParametersViewInfo{
    pub id: String,
    pub hash: String,
    pub state: WindowState,

    // Прочие поля
    //#[serde(flatten)]
    //other: HashMap<String, Value>
}

// https://serde.rs/enum-representations.html
// https://api.slack.com/reference/interaction-payloads
#[derive(Deserialize, Serialize)]
#[serde(tag = "type")] // Вариант enum будет выбираться по полю type, значения переименовываются
pub enum WindowParametersPayload{
    #[serde(rename = "view_submission")]
    Submit{
        trigger_id: String,
        //response_url: Option<String>,
        view: WindowParametersViewInfo,
    },
    
    #[serde(rename = "block_actions")]
    Action{
        trigger_id: String,
        //response_url: Option<String>,
        view: WindowParametersViewInfo,
        actions: Vec<Value>,
    }

    // pub user: HashMap<String, serde_json::Value>,
    // pub view: HashMap<String, Value>,

    // Прочие поля
    // #[serde(flatten)]
    // other: HashMap<String, Value>
}

impl fmt::Debug for WindowParametersPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(serde_json::to_string_pretty(self)
            .unwrap()
            .as_str())
    }
}

#[derive(Deserialize, Debug)]
pub struct WindowParameters{
    pub payload: String
}
