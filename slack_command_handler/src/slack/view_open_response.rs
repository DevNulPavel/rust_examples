use std::{
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
pub struct ViewInfo{
    pub id: String,
    pub hash: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ViewOpenErrorInfo{
    error: String,
    response_metadata: HashMap<String, Value>
}

// https://serde.rs/enum-representations.html
//https://api.slack.com/methods/views.open#response
//#[serde(tag = "ok")] // Вариант enum будет выбираться по полю type, значения переименовываются
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum ViewOpenResponse{
    //#[serde(rename = "true")]
    Ok{
        //pub ok: bool,
        view: ViewInfo
    },
    
    //#[serde(rename = "false")]
    Error(ViewOpenErrorInfo)

    // pub user: HashMap<String, serde_json::Value>,
    // pub view: HashMap<String, Value>,

    // Прочие поля
    // #[serde(flatten)]
    // other: HashMap<String, Value>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ViewUpdateErrorInfo{
    error: String
}

// https://serde.rs/enum-representations.html
//https://api.slack.com/methods/views.update
//#[serde(tag = "ok")] // Вариант enum будет выбираться по полю type, значения переименовываются
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum ViewUpdateResponse{
    //#[serde(rename = "true")]
    Ok{
        //pub ok: bool,
        view: ViewInfo
    },
    
    //#[serde(rename = "false")]
    Error(ViewUpdateErrorInfo)

    // pub user: HashMap<String, serde_json::Value>,
    // pub view: HashMap<String, Value>,

    // Прочие поля
    // #[serde(flatten)]
    // other: HashMap<String, Value>
}