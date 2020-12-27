use std::{
    collections::{
        HashMap
    }
};
// use async_trait::{
//     async_trait
// };
// use actix_web::{
//     web::{
//         Data
//     }
// };
use serde_json::{
    Value
};
use serde::{
    Deserialize
};
use crate::{
    session::{
        WindowSession
    },
    // ApplicationData
};
use super::{
    error::{
        SlackError,
        ViewUpdateErrorInfo
    },
    request_builder::{
        SlackRequestBuilder
    }
};

////////////////////////////////////////////////////////////////

// https://api.slack.com/reference/interaction-payloads/views#view_submission
// https://api.slack.com/reference/surfaces/views
#[derive(Deserialize, Debug)]
pub struct ViewInfo{
    id: String,
    hash: String,
    callback_id: Option<String>,
    //private_metadata: Option<String>,
    state: Option<HashMap<String, Value>>
}

impl ViewInfo {
    pub fn get_id(&self) -> &str{
        &self.id
    }
    pub fn get_state(&self) -> &Option<HashMap<String, Value>>{
        &self.state
    }
}

////////////////////////////////////////////////////////////////

// #[async_trait]
pub trait ViewActionHandler: Send {
    fn update_info(&mut self, new_info: ViewInfo);
    fn get_view(&self) -> &View;
    fn on_submit(self: Box<Self>, session: WindowSession);
    fn on_update(&self);
    fn on_close(self: Box<Self>, session: WindowSession);
}

////////////////////////////////////////////////////////////////

pub struct View {
    client: SlackRequestBuilder,
    info: ViewInfo
}

impl View {
    pub fn new(client: SlackRequestBuilder, info: ViewInfo) -> View{
        View{
            client,
            info
        }
    }

    pub fn get_info(&self) -> &ViewInfo{
        return &self.info;
    }

    pub fn set_info(&mut self, new_info: ViewInfo){
        self.info = new_info;
    }

    pub fn get_id(&self) -> &str{
        return self.info.get_id();
    }

    pub async fn update_view(&mut self, view_json: Value) -> Result<(), SlackError>{
        // https://serde.rs/enum-representations.html
        // https://api.slack.com/methods/views.update
        #[derive(Deserialize, Debug)]
        #[serde(untagged)]
        pub enum ViewUpdateResponse{
            Ok{ view: ViewInfo },
            Error(ViewUpdateErrorInfo)
        }

        // TODO: Снизить область видимости
        let window = serde_json::json!({
            "view_id": self.info.id,
            "hash": self.info.hash,
            "view": view_json
        });

        let response = self.client
            .build_post_request("https://slack.com/api/views.update")
            .header("Content-type", "application/json")
            .body(serde_json::to_string(&window).unwrap())
            .send()
            .await
            .map_err(|err|{
                SlackError::RequestErr(err)
            })?
            .json::<ViewUpdateResponse>()
            .await
            .map_err(|err|{
                SlackError::JsonParseError(err)
            })?;

        match response {
            ViewUpdateResponse::Ok{view} => {
                self.info = view;
                Ok(())
            },
            ViewUpdateResponse::Error(err) => {
                Err(SlackError::UpdateError(err))
            }
        }
    }
}