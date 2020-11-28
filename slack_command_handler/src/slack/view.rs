use std::{
    collections::{
        HashMap
    }
};
use actix_web::{
    client::{
        Client
    }
};
use serde_json::{
    Value
};
use serde::{
    Deserialize
};
use super::{
    error::{
        SlackViewError,
        ViewUpdateErrorInfo
    }
};


// https://api.slack.com/reference/interaction-payloads/views#view_submission
// https://api.slack.com/reference/surfaces/views
#[derive(Deserialize, Debug)]
pub struct ViewInfo{
    id: String,
    hash: String,
    callback_id: Option<String>,
    private_metadata: Option<String>,
    state: Option<HashMap<String, Value>>
}

pub struct View<'a> {
    client: &'a Client,
    info: ViewInfo
}

impl<'a> View<'a> {
    pub fn new(client: &'a Client, info: ViewInfo) -> View<'a>{
        View{
            client,
            info
        }
    }

    pub fn get_info(&self) -> &ViewInfo{
        return &self.info;
    }

    pub async fn update_view(self, view_json: Value) -> Result<View<'a>, SlackViewError>{
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
            .post("https://slack.com/api/views.update")
            .send_body(serde_json::to_string(&window).unwrap())
            .await?
            .json::<ViewUpdateResponse>()
            .await?;

        match response {
            ViewUpdateResponse::Ok{view} => {
                Ok(View::new(self.client, view))
            },
            ViewUpdateResponse::Error(err) => {
                Err(SlackViewError::UpdateError(err))
            }
        }
    }
}